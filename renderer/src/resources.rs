use crate::basic_renderer;
use crate::convert::*;
use crate::keyboard_model;
use gl_typed as gl;
use gl_typed::convert::*;
use std::mem;
use std::path::Path;

#[allow(unused)]
pub struct Resources {
    pub models: Vec<tobj::Model>,
    pub materials: Vec<tobj::Material>,
    pub diffuse_textures: Vec<gl::TextureName>,
    pub vaos: Vec<gl::VertexArrayName>,
    pub vbs: Vec<gl::BufferName>,
    pub ebs: Vec<gl::BufferName>,
    pub element_counts: Vec<usize>,
    pub key_indices: Vec<keyboard_model::UncheckedIndex>,
}

impl Resources {
    pub fn new<P: AsRef<Path>>(
        gl: &gl::Gl,
        resource_dir: P,
        renderer: &basic_renderer::Renderer,
    ) -> Self {
        let resource_dir = resource_dir.as_ref();

        let room_obj = tobj::load_obj(&resource_dir.join("room.obj"));
        let (mut room_models, mut room_materials) = room_obj.unwrap();

        let keyboard_obj = tobj::load_obj(&resource_dir.join("keyboard.obj"));
        let (mut keyboard_models, mut keyboard_materials) = keyboard_obj.unwrap();

        for model in keyboard_models.iter_mut() {
            // Offset keyboard model material ids.
            if let Some(ref mut id) = model.mesh.material_id {
                *id += room_materials.len();
            }
        }

        let mut models: Vec<tobj::Model> =
            Vec::with_capacity(room_models.len() + keyboard_models.len());

        models.append(&mut room_models);
        models.append(&mut keyboard_models);

        let mut materials: Vec<tobj::Material> =
            Vec::with_capacity(room_materials.len() + keyboard_materials.len());
        materials.append(&mut room_materials);
        materials.append(&mut keyboard_materials);

        let vaos = unsafe {
            let mut names = Vec::with_capacity(models.len());
            names.set_len(models.len());
            gl.gen_vertex_arrays(&mut names);
            names.try_transmute_each().unwrap()
        };

        let vbs = unsafe {
            let mut names = Vec::with_capacity(models.len());
            names.set_len(models.len());
            gl.gen_buffers(&mut names);
            names.try_transmute_each().unwrap()
        };

        let ebs = unsafe {
            let mut names = Vec::with_capacity(models.len());
            names.set_len(models.len());
            gl.gen_buffers(&mut names);
            names.try_transmute_each().unwrap()
        };

        let element_counts: Vec<usize> = models
            .iter()
            .map(|model| model.mesh.indices.len())
            .collect();

        let key_indices: Vec<keyboard_model::UncheckedIndex> = models
            .iter()
            .map(|model| model_name_to_keyboard_index(&model.name))
            .collect();

        for model in models.iter_mut() {
            if model.mesh.normals.len() == 0 {
                model.mesh.normals = polygen::compute_normals_tris(
                    (&model.mesh.indices[..]).unflatten(),
                    (&model.mesh.positions[..]).unflatten(),
                )
                .flatten()
            }
        }

        for (i, model) in models.iter().enumerate() {
            let vao = vaos[i];
            let vb = vbs[i];
            let eb = ebs[i];

            unsafe {
                let ver_pos_size = mem::size_of_val(&model.mesh.positions[..]);
                let ver_nor_size = mem::size_of_val(&model.mesh.normals[..]);
                let tex_pos_size = mem::size_of_val(&model.mesh.texcoords[..]);

                let ver_pos_offset = 0;
                let ver_nor_offset = ver_pos_offset + ver_pos_size;
                let tex_pos_offset = ver_nor_offset + ver_nor_size;

                gl.bind_vertex_array(vao);
                gl.bind_buffer(gl::ARRAY_BUFFER, vb);
                gl.buffer_reserve(
                    gl::ARRAY_BUFFER,
                    ver_pos_size + ver_nor_size + tex_pos_size,
                    gl::STATIC_DRAW,
                );
                gl.buffer_sub_data(gl::ARRAY_BUFFER, ver_pos_offset, &model.mesh.positions[..]);
                gl.buffer_sub_data(gl::ARRAY_BUFFER, ver_nor_offset, &model.mesh.normals[..]);
                gl.buffer_sub_data(gl::ARRAY_BUFFER, tex_pos_offset, &model.mesh.texcoords[..]);

                gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, eb);
                gl.buffer_data(
                    gl::ELEMENT_ARRAY_BUFFER,
                    &model.mesh.indices,
                    gl::STATIC_DRAW,
                );

                // AOS layout.
                if let Some(loc) = renderer.vs_ver_pos_loc.into() {
                    gl.vertex_attrib_pointer(
                        loc,
                        3,
                        gl::FLOAT,
                        gl::FALSE,
                        mem::size_of::<[f32; 3]>(),
                        ver_pos_offset,
                    );

                    gl.enable_vertex_attrib_array(loc);
                }

                if let Some(loc) = renderer.vs_ver_nor_loc.into() {
                    gl.vertex_attrib_pointer(
                        loc,
                        3,
                        gl::FLOAT,
                        gl::FALSE,
                        mem::size_of::<[f32; 3]>(),
                        ver_nor_offset,
                    );

                    gl.enable_vertex_attrib_array(loc);
                }

                if let Some(loc) = renderer.vs_tex_pos_loc.into() {
                    gl.vertex_attrib_pointer(
                        loc,
                        2,
                        gl::FLOAT,
                        gl::FALSE,
                        mem::size_of::<[f32; 2]>(),
                        tex_pos_offset,
                    );

                    gl.enable_vertex_attrib_array(loc);
                }

                gl.unbind_vertex_array();
                gl.unbind_buffer(gl::ARRAY_BUFFER);
                gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);
            }
        }

        let ebs = unsafe {
            let mut names = Vec::with_capacity(models.len());
            names.set_len(models.len());
            gl.gen_buffers(&mut names);
            names.try_transmute_each().unwrap()
        };

        let diffuse_textures = unsafe {
            let mut names = Vec::with_capacity(materials.len());
            names.set_len(materials.len());
            gl.gen_textures(&mut names);
            names.try_transmute_each().unwrap()
        };

        for (i, material) in materials.iter().enumerate() {
            let path = resource_dir.join(&material.diffuse_texture);

            {
                let img = image::open(path).unwrap().flipv().to_rgba();
                unsafe {
                    gl.bind_texture(gl::TEXTURE_2D, diffuse_textures[i]);
                    gl.tex_image_2d(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA8,
                        img.width() as i32,
                        img.height() as i32,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        img.as_ptr() as *const std::os::raw::c_void,
                    );
                    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
                    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
                }
            }
        }

        Resources {
            models,
            materials,
            diffuse_textures,
            vaos,
            vbs,
            ebs,
            element_counts,
            key_indices,
        }
    }
}

fn model_name_to_keyboard_index(name: &str) -> keyboard_model::UncheckedIndex {
    let code = match name {
        "Key_RIGHT_CONTROL_Key_LP.008" => Some(glutin::VirtualKeyCode::LControl),
        "Key_MENU_Key_LP.009" => Some(glutin::VirtualKeyCode::Apps),
        "Key_RIGHT_SUPER_Key_LP.010" => Some(glutin::VirtualKeyCode::RWin),
        "Key_RIGHT_ALT_Key_LP.011" => Some(glutin::VirtualKeyCode::RAlt),
        "Key_ESCAPE_Key_LP.012" => Some(glutin::VirtualKeyCode::Escape),
        "Key_LEFT_CONTROL_Key_LP.013" => Some(glutin::VirtualKeyCode::LControl),
        "Key_SUPER_Key_LP.014" => Some(glutin::VirtualKeyCode::LWin),
        "Key_ALT_Key_LP.015" => Some(glutin::VirtualKeyCode::LAlt),
        "Key_SPACE_Key_LP.003" => Some(glutin::VirtualKeyCode::Space),
        "Key_LEFT_SHIFT_Key_LP.004" => Some(glutin::VirtualKeyCode::LShift),
        "Key_CAPSLOCK_Key_LP.016" => Some(glutin::VirtualKeyCode::Capital),
        "Key_TAB_Key_LP.017" => Some(glutin::VirtualKeyCode::Tab),
        "Key_RSHIFT_Key_LP.005" => Some(glutin::VirtualKeyCode::RShift),
        "Key_ENTER_Key_LP.018" => Some(glutin::VirtualKeyCode::Return),
        "Key_\\_Key_LP.019" => Some(glutin::VirtualKeyCode::Backslash),
        "Key_BACKSPACE_Key_LP.020" => Some(glutin::VirtualKeyCode::Back),
        "Key_NUM_ENTER_Key_LP.021" => Some(glutin::VirtualKeyCode::NumpadEnter),
        "Key_NUM_ADD_Key_LP.006" => Some(glutin::VirtualKeyCode::Add),
        "Key_NUM_MIN_Key_LP.022" => Some(glutin::VirtualKeyCode::Subtract),
        "Key_NUM_0_Key_LP.007" => Some(glutin::VirtualKeyCode::Numpad0),
        "Key_NUM_DOT_Key_LP.023" => Some(glutin::VirtualKeyCode::NumpadComma),
        "Key_NUM_3_Key_LP.024" => Some(glutin::VirtualKeyCode::Numpad3),
        "Key_NUM_2_Key_LP.025" => Some(glutin::VirtualKeyCode::Numpad2),
        "Key_NUM_1_Key_LP.026" => Some(glutin::VirtualKeyCode::Numpad1),
        "Key_NUM_4_Key_LP.027" => Some(glutin::VirtualKeyCode::Numpad4),
        "Key_NUM_5_Key_LP.028" => Some(glutin::VirtualKeyCode::Numpad5),
        "Key_NUM_6_Key_LP.029" => Some(glutin::VirtualKeyCode::Numpad6),
        "Key_NUM_9_Key_LP.030" => Some(glutin::VirtualKeyCode::Numpad9),
        "Key_NUM_8_Key_LP.031" => Some(glutin::VirtualKeyCode::Numpad8),
        "Key_NUM_7_Key_LP.032" => Some(glutin::VirtualKeyCode::Numpad7),
        "Key_NUM_LCK_Key_LP.033" => Some(glutin::VirtualKeyCode::Numlock),
        "Key_NUM_DIV_Key_LP.034" => Some(glutin::VirtualKeyCode::Divide),
        "Key_NUM_MUL_Key_LP.035" => Some(glutin::VirtualKeyCode::Multiply),
        "Key_T4_Key_LP.036" => None, // TODO
        "Key_T3_Key_LP.037" => None, // TODO
        "Key_T2_Key_LP.038" => None, // TODO
        "Key_T1_Key_LP.039" => None, // TODO
        "Key_Up_Key_LP.040" => Some(glutin::VirtualKeyCode::Up),
        "Key_Left_Key_LP.041" => Some(glutin::VirtualKeyCode::Left),
        "Key_Down_Key_LP.042" => Some(glutin::VirtualKeyCode::Down),
        "Key_Left.001_Key_LP.043" => Some(glutin::VirtualKeyCode::Right), // FIXME: Inconsistent
        "Key_PGDN_Key_LP.044" => Some(glutin::VirtualKeyCode::PageDown),
        "Key_END_Key_LP.045" => Some(glutin::VirtualKeyCode::End),
        "Key_DEL_Key_LP.046" => Some(glutin::VirtualKeyCode::Delete),
        "Key_INS_Key_LP.047" => Some(glutin::VirtualKeyCode::Insert),
        "Key_Home_Key_LP.048" => Some(glutin::VirtualKeyCode::Home),
        "Key_PGUP_Key_LP.049" => Some(glutin::VirtualKeyCode::PageUp),
        "Key_PAUSE_Key_LP.050" => Some(glutin::VirtualKeyCode::Pause),
        "Key_SCRL_Key_LP.051" => Some(glutin::VirtualKeyCode::Scroll),
        "Key_PRNT_Key_LP.052" => Some(glutin::VirtualKeyCode::Snapshot),
        "Key_F12_Key_LP.053" => Some(glutin::VirtualKeyCode::F12),
        "Key_F12.001_Key_LP.054" => Some(glutin::VirtualKeyCode::F11),
        "Key_F10_Key_LP.055" => Some(glutin::VirtualKeyCode::F10),
        "Key_F9_Key_LP.056" => Some(glutin::VirtualKeyCode::F9),
        "Key_F8_Key_LP.057" => Some(glutin::VirtualKeyCode::F8),
        "Key_F7_Key_LP.058" => Some(glutin::VirtualKeyCode::F7),
        "Key_F6_Key_LP.059" => Some(glutin::VirtualKeyCode::F6),
        "Key_F5_Key_LP.060" => Some(glutin::VirtualKeyCode::F5),
        "Key_F4_Key_LP.061" => Some(glutin::VirtualKeyCode::F4),
        "Key_F3_Key_LP.062" => Some(glutin::VirtualKeyCode::F3),
        "Key_F2_Key_LP.063" => Some(glutin::VirtualKeyCode::F2),
        "Key_F1_Key_LP.064" => Some(glutin::VirtualKeyCode::F1),
        "Key_=_Key_LP.065" => Some(glutin::VirtualKeyCode::Equals),
        "Key_-_Key_LP.066" => Some(glutin::VirtualKeyCode::Minus),
        "Key_0_Key_LP.067" => Some(glutin::VirtualKeyCode::Key0),
        "Key_9_Key_LP.068" => Some(glutin::VirtualKeyCode::Key9),
        "Key_8_Key_LP.069" => Some(glutin::VirtualKeyCode::Key8),
        "Key_7_Key_LP.070" => Some(glutin::VirtualKeyCode::Key7),
        "Key_6_Key_LP.071" => Some(glutin::VirtualKeyCode::Key6),
        "Key_5_Key_LP.072" => Some(glutin::VirtualKeyCode::Key5),
        "Key_4_Key_LP.073" => Some(glutin::VirtualKeyCode::Key4),
        "Key_3_Key_LP.074" => Some(glutin::VirtualKeyCode::Key3),
        "Key_2_Key_LP.075" => Some(glutin::VirtualKeyCode::Key2),
        "Key_`_Key_LP.076" => Some(glutin::VirtualKeyCode::Apostrophe),
        "Key_1_Key_LP.077" => Some(glutin::VirtualKeyCode::Key1),
        "Key_/_Key_LP.002" => Some(glutin::VirtualKeyCode::Slash),
        "Key_._Key_LP.001" => Some(glutin::VirtualKeyCode::Period),
        "Key_,_Key_LP.078" => Some(glutin::VirtualKeyCode::Comma),
        "Key_M_Key_LP.079" => Some(glutin::VirtualKeyCode::M),
        "Key_N_Key_LP.080" => Some(glutin::VirtualKeyCode::N),
        "Key_B_Key_LP.081" => Some(glutin::VirtualKeyCode::B),
        "Key_V_Key_LP.082" => Some(glutin::VirtualKeyCode::V),
        "Key_C_Key_LP.083" => Some(glutin::VirtualKeyCode::C),
        "Key_X_Key_LP.084" => Some(glutin::VirtualKeyCode::X),
        "Key_Z_Key_LP.085" => Some(glutin::VirtualKeyCode::Z),
        "Key_'_Key_LP.086" => Some(glutin::VirtualKeyCode::Apostrophe),
        "Key_;_Key_LP.087" => Some(glutin::VirtualKeyCode::Semicolon),
        "Key_L_Key_LP.088" => Some(glutin::VirtualKeyCode::L),
        "Key_K_Key_LP.089" => Some(glutin::VirtualKeyCode::K),
        "Key_J_Key_LP.090" => Some(glutin::VirtualKeyCode::J),
        "Key_H_Key_LP.091" => Some(glutin::VirtualKeyCode::H),
        "Key_G_Key_LP.092" => Some(glutin::VirtualKeyCode::G),
        "Key_F_Key_LP.093" => Some(glutin::VirtualKeyCode::F),
        "Key_D_Key_LP.094" => Some(glutin::VirtualKeyCode::D),
        "Key_S_Key_LP.095" => Some(glutin::VirtualKeyCode::S),
        "Key_A_Key_LP.096" => Some(glutin::VirtualKeyCode::A),
        "Key_]_Key_LP.097" => Some(glutin::VirtualKeyCode::RBracket),
        "Key_[_Key_LP.098" => Some(glutin::VirtualKeyCode::LBracket),
        "Key_P_Key_LP.099" => Some(glutin::VirtualKeyCode::P),
        "Key_O_Key_LP.100" => Some(glutin::VirtualKeyCode::O),
        "Key_I_Key_LP.101" => Some(glutin::VirtualKeyCode::I),
        "Key_U_Key_LP.102" => Some(glutin::VirtualKeyCode::U),
        "Key_Y_Key_LP.103" => Some(glutin::VirtualKeyCode::Y),
        "Key_T_Key_LP.104" => Some(glutin::VirtualKeyCode::T),
        "Key_R_Key_LP.105" => Some(glutin::VirtualKeyCode::R),
        "Key_E_Key_LP.106" => Some(glutin::VirtualKeyCode::E),
        "Key_W_Key_LP.107" => Some(glutin::VirtualKeyCode::W),
        "Key_Q_Key_LP" => Some(glutin::VirtualKeyCode::Q),
        "Base_Cube.001" => None,
        _ => None, // Unknown model in obj file.
    };

    match code {
        Some(code) => keyboard_model::Index::from_code(code).into(),
        None => keyboard_model::Index::INVALID,
    }
}
