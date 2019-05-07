use crate::convert::*;
use crate::keyboard_model;
use crate::shader_defines;
use cgmath::*;
use gl_typed as gl;
use gl_typed::convert::*;
use std::mem;
use std::path::Path;

#[allow(unused)]
pub struct Resources {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<tobj::Material>,
    pub diffuse_textures: Vec<gl::TextureName>,
    pub diffuse_dimensions: Vec<[f32; 2]>,
    pub normal_textures: Vec<gl::TextureName>,
    pub normal_dimensions: Vec<[f32; 2]>,
    pub vaos: Vec<gl::VertexArrayName>,
    pub vbs: Vec<gl::BufferName>,
    pub ebs: Vec<gl::BufferName>,
    pub element_counts: Vec<usize>,
    pub key_indices: Vec<keyboard_model::UncheckedIndex>,
}

pub struct Mesh {
    pub name: String,
    /// Triangle indices.
    pub triangles: Vec<[u32; 3]>,

    /// Position in object space.
    pub pos_in_obj: Vec<[f32; 3]>,

    /// Position in texture space.
    pub pos_in_tex: Vec<[f32; 2]>,

    /// Normal in object space.
    pub nor_in_obj: Vec<[f32; 3]>,

    /// Tangent in object space.
    pub tan_in_obj: Vec<[f32; 3]>,

    /// Derpiederp.
    pub translate: Vector3<f32>,

    /// Material id.
    pub material_id: Option<u32>,
}

impl Resources {
    pub fn new<P: AsRef<Path>>(gl: &gl::Gl, resource_dir: P) -> Self {
        let resource_dir = resource_dir.as_ref();

        let objs: Vec<(String, Vec<tobj::Model>, Vec<tobj::Material>)> =
            // ["two_planes.obj", "bunny.obj"]
                ["sponza/sponza.obj", "keyboard.obj"]
            // ["shadow_test.obj"]
                .into_iter()
                .map(|&rel_file_path| {
                    let file_path = &resource_dir.join(rel_file_path);
                    println!("Loading {:?}.", file_path.display());
                    let mut obj = tobj::load_obj(&file_path).unwrap();
                    let vertex_count: usize =
                        obj.0.iter().map(|model| model.mesh.indices.len()).sum();
                    println!(
                        "Loaded {:?} with {} vertices.",
                        file_path.display(),
                        vertex_count
                    );
                    let file_dir = file_path.parent().unwrap();
                    for material in obj.1.iter_mut() {
                        if !material.ambient_texture.is_empty() {
                            material.ambient_texture = String::from(
                                file_dir.join(&material.ambient_texture).to_string_lossy(),
                            );
                        }
                        if !material.diffuse_texture.is_empty() {
                            material.diffuse_texture = String::from(
                                file_dir.join(&material.diffuse_texture).to_string_lossy(),
                            );
                        }
                        if !material.specular_texture.is_empty() {
                            material.specular_texture = String::from(
                                file_dir.join(&material.specular_texture).to_string_lossy(),
                            );
                        }
                        if !material.normal_texture.is_empty() {
                            material.normal_texture = String::from(
                                file_dir.join(&material.normal_texture).to_string_lossy(),
                            );
                        }
                        if !material.dissolve_texture.is_empty() {
                            material.dissolve_texture = String::from(
                                file_dir.join(&material.dissolve_texture).to_string_lossy(),
                            );
                        }
                    }
                    (String::from(rel_file_path), obj.0, obj.1)
                })
                .collect();

        let material_offsets: Vec<u32> = objs
            .iter()
            .scan(0, |sum, (_, _, ref materials)| {
                let offset = *sum;
                *sum += materials.len() as u32;
                Some(offset)
            })
            .collect();

        let mut meshes: Vec<Mesh> = Vec::with_capacity(objs.iter().map(|(_, ref models, _)| models.len()).sum());
        let mut materials: Vec<tobj::Material> =
            Vec::with_capacity(objs.iter().map(|(_, _, ref materials)| materials.len()).sum());

        for (i, (obj_path, obj_models, obj_materials)) in objs.into_iter().enumerate() {
            let material_offset = material_offsets[i];

            meshes.extend(obj_models.into_iter().map(|model| {
                let tobj::Model { mesh, name } = model;
                let tobj::Mesh {
                    positions,
                    normals: _normals,
                    texcoords,
                    indices,
                    material_id,
                } = mesh;

                let triangles: Vec<[u32; 3]> = indices.unflatten();
                let pos_in_obj: Vec<[f32; 3]> = positions.unflatten();

                let pos_in_tex: Vec<[f32; 2]> = texcoords.unflatten();
                let nor_in_obj = polygen::compute_normals(&triangles, &pos_in_obj);
                let tan_in_obj = if pos_in_tex.is_empty() {
                    Vec::new()
                } else {
                    polygen::compute_tangents(&triangles, &pos_in_obj, &pos_in_tex)
                };
                let material_id = material_id.map(|id| id as u32 + material_offset);

                Mesh {
                    name,
                    triangles,
                    pos_in_obj,
                    pos_in_tex,
                    nor_in_obj,
                    tan_in_obj,
                    translate: if obj_path == "keyboard.obj" {
                        Vector3::new(0.0, 0.3, 0.0)
                    } else {
                        Vector3::zero()
                    },
                    material_id,
                }
            }));

            materials.extend(obj_materials.into_iter());
        }

        let vaos = unsafe {
            let mut names = Vec::with_capacity(meshes.len());
            names.set_len(meshes.len());
            gl.gen_vertex_arrays(&mut names);
            names.try_transmute_each().unwrap()
        };

        let vbs = unsafe {
            let mut names = Vec::with_capacity(meshes.len());
            names.set_len(meshes.len());
            gl.gen_buffers(&mut names);
            names.try_transmute_each().unwrap()
        };

        let ebs = unsafe {
            let mut names = Vec::with_capacity(meshes.len());
            names.set_len(meshes.len());
            gl.gen_buffers(&mut names);
            names.try_transmute_each().unwrap()
        };

        let element_counts: Vec<usize> = meshes.iter().map(|mesh| mesh.triangles.len() * 3).collect();

        let key_indices: Vec<keyboard_model::UncheckedIndex> = meshes
            .iter()
            .map(|mesh| model_name_to_keyboard_index(&mesh.name))
            .collect();

        for (i, mesh) in meshes.iter().enumerate() {
            let vao = vaos[i];
            let vb = vbs[i];
            let eb = ebs[i];

            unsafe {
                let pos_in_obj_size = mem::size_of_val(&mesh.pos_in_obj[..]);
                let pos_in_tex_size = mem::size_of_val(&mesh.pos_in_tex[..]);
                let nor_in_obj_size = mem::size_of_val(&mesh.nor_in_obj[..]);
                let tan_in_obj_size = mem::size_of_val(&mesh.tan_in_obj[..]);

                let pos_in_obj_offset = 0;
                let pos_in_tex_offset = pos_in_obj_offset + pos_in_obj_size;
                let nor_in_obj_offset = pos_in_tex_offset + pos_in_tex_size;
                let tan_in_obj_offset = nor_in_obj_offset + nor_in_obj_size;
                let total_size = tan_in_obj_offset + tan_in_obj_size;

                gl.bind_vertex_array(vao);
                gl.bind_buffer(gl::ARRAY_BUFFER, vb);
                gl.buffer_reserve(gl::ARRAY_BUFFER, total_size, gl::STATIC_DRAW);
                gl.buffer_sub_data(gl::ARRAY_BUFFER, pos_in_obj_offset, (&mesh.pos_in_obj[..]).flatten());
                gl.buffer_sub_data(gl::ARRAY_BUFFER, pos_in_tex_offset, (&mesh.pos_in_tex[..]).flatten());
                gl.buffer_sub_data(gl::ARRAY_BUFFER, nor_in_obj_offset, (&mesh.nor_in_obj[..]).flatten());
                gl.buffer_sub_data(gl::ARRAY_BUFFER, tan_in_obj_offset, (&mesh.tan_in_obj[..]).flatten());

                gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, eb);
                // NOTE: Add this to have renderdoc show the BufferData calls.
                // See https://github.com/baldurk/renderdoc/issues/1307.
                // gl.buffer_reserve(
                //     gl::ELEMENT_ARRAY_BUFFER,
                //     std::mem::size_of_val(&mesh.triangles[..]),
                //     gl::STATIC_DRAW,
                // );
                gl.buffer_data(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (&mesh.triangles[..]).flatten(),
                    gl::STATIC_DRAW,
                );

                // AOS layout.
                gl.vertex_attrib_pointer(
                    shader_defines::VS_POS_IN_OBJ_LOC,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<[f32; 3]>(),
                    pos_in_obj_offset,
                );

                gl.enable_vertex_attrib_array(shader_defines::VS_POS_IN_OBJ_LOC);

                gl.vertex_attrib_pointer(
                    shader_defines::VS_POS_IN_TEX_LOC,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<[f32; 2]>(),
                    pos_in_tex_offset,
                );

                gl.enable_vertex_attrib_array(shader_defines::VS_POS_IN_TEX_LOC);

                gl.vertex_attrib_pointer(
                    shader_defines::VS_NOR_IN_OBJ_LOC,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<[f32; 3]>(),
                    nor_in_obj_offset,
                );

                gl.enable_vertex_attrib_array(shader_defines::VS_NOR_IN_OBJ_LOC);

                gl.vertex_attrib_pointer(
                    shader_defines::VS_TAN_IN_OBJ_LOC,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<[f32; 3]>(),
                    tan_in_obj_offset,
                );

                gl.enable_vertex_attrib_array(shader_defines::VS_TAN_IN_OBJ_LOC);

                gl.unbind_vertex_array();
                gl.unbind_buffer(gl::ARRAY_BUFFER);
                gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);
            }
        }

        let diffuse_textures = unsafe {
            let mut names = Vec::with_capacity(materials.len());
            names.set_len(materials.len());
            gl.gen_textures(&mut names);
            names.try_transmute_each().unwrap()
        };

        let mut diffuse_dimensions = Vec::with_capacity(materials.len());

        let normal_textures = unsafe {
            let mut names = Vec::with_capacity(materials.len());
            names.set_len(materials.len());
            gl.gen_textures(&mut names);
            names.try_transmute_each().unwrap()
        };

        let mut normal_dimensions = Vec::with_capacity(materials.len());

        for (i, material) in materials.iter().enumerate() {
            let mut dimensions = [0.0, 0.0];

            if !material.diffuse_texture.is_empty() {
                let file_path = resource_dir.join(&material.diffuse_texture);
                println!("Loading diffuse texture {:?}", &file_path);
                match image::open(&file_path) {
                    Ok(img) => {
                        let img = img.flipv().to_rgba();

                        dimensions = [img.width() as f32, img.height() as f32];

                        unsafe {
                            loop {
                                if gl.get_error() == 0 {
                                    break;
                                }
                            }

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
                            gl.generate_mipmap(gl::TEXTURE_2D);
                            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
                            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR);

                            loop {
                                let error = gl.get_error();
                                if error == 0 {
                                    break;
                                }
                                eprintln!("OpenGL error {}", error);
                            }
                        }
                        println!("Loaded diffuse texture {:?}", &file_path);
                    }
                    Err(err) => {
                        eprintln!("Failed to load diffuse texture {:?}: {}", &file_path, err);
                    }
                }
            }

            diffuse_dimensions.push(dimensions);

            let mut dimensions = [0.0, 0.0];

            println!("{}", material.normal_texture);
            if !material.normal_texture.is_empty() {
                let file_path = resource_dir.join(&material.normal_texture);
                println!("Loading normal texture {:?}", &file_path);
                match image::open(&file_path) {
                    Ok(img) => {
                        let img = img.flipv().to_rgba();
                        unsafe {
                            loop {
                                if gl.get_error() == 0 {
                                    break;
                                }
                            }

                            dimensions = [img.width() as f32, img.height() as f32];

                            gl.bind_texture(gl::TEXTURE_2D, normal_textures[i]);
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
                            gl.generate_mipmap(gl::TEXTURE_2D);
                            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
                            gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR);

                            loop {
                                let error = gl.get_error();
                                if error == 0 {
                                    break;
                                }
                                eprintln!("OpenGL error {}", error);
                            }
                        }
                        println!("Loaded normal texture {:?}", &file_path);
                    }
                    Err(err) => {
                        eprintln!("Failed to load normal texture {:?}: {}", &file_path, err);
                    }
                }
            }

            normal_dimensions.push(dimensions);
        }

        Resources {
            meshes,
            materials,
            diffuse_textures,
            diffuse_dimensions,
            normal_textures,
            normal_dimensions,
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
