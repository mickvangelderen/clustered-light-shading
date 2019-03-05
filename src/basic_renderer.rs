use super::keyboard_model;
use super::World;
use cgmath::*;
use gl_typed as gl;
use std::mem;

const VS_SRC: &'static [u8] = b"
#version 400 core

uniform mat4 pos_from_wld_to_clp;
uniform float highlight;

in vec3 vs_ver_pos;
in vec2 vs_tex_pos;
out vec2 fs_tex_pos;

void main() {
    mat4 pos_from_obj_to_wld = mat4(
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, -0.02*highlight, 0.0, 1.0
    );

    gl_Position = pos_from_wld_to_clp*pos_from_obj_to_wld*vec4(vs_ver_pos, 1.0);
    fs_tex_pos = vs_tex_pos;
}\0";

const FS_SRC: &'static [u8] = b"
#version 400

uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;
uniform float highlight;

in vec2 fs_tex_pos;
out vec4 frag_color;

void main() {
    vec4 d = texture(diffuse_sampler, fs_tex_pos);
    frag_color = vec4(mix(d.rgb, vec3(1.0, 1.0, 1.0), highlight), d.a);
}
\0";

unsafe fn recompile_shader(
    gl: &gl::Gl,
    name: &mut gl::ShaderName,
    source: &[u8],
) -> Result<(), String> {
    gl.shader_source(name, &[source]);
    gl.compile_shader(name);
    let status = gl.get_shaderiv_move(name, gl::COMPILE_STATUS);
    if status == gl::ShaderCompileStatus::Compiled.into() {
        Ok(())
    } else {
        let log = gl.get_shader_info_log_move(&name);
        Err(String::from_utf8(log).unwrap())
    }
}

pub struct Renderer {
    program: gl::ProgramName,
    pos_from_wld_to_clp_loc: gl::UniformLocation<[[f32; 4]; 4]>,
    highlight_loc: gl::UniformLocation<f32>,
    diffuse_sampler_loc: gl::UniformLocation<i32>,
    // #[allow(unused)]
    // normal_sampler_loc: gl::UniformLocation<i32>,
    diffuse_textures: Vec<gl::TextureName>,
    vaos: Vec<gl::VertexArrayName>,
    #[allow(unused)]
    vbs: Vec<gl::BufferName>,
    #[allow(unused)]
    ebs: Vec<gl::BufferName>,
    element_counts: Vec<usize>,
    key_indices: Vec<keyboard_model::UncheckedIndex>,
}

pub struct Parameters<'a, N: 'a>
where
    N: gl::MaybeDefaultFramebufferName,
{
    pub framebuffer: &'a N,
    pub width: i32,
    pub height: i32,
    pub pos_from_cam_to_clp: Matrix4<f32>,
}

impl Renderer {
    pub unsafe fn render<'a, N>(&self, gl: &gl::Gl, params: &Parameters<'a, N>, world: &World)
    where
        N: gl::MaybeDefaultFramebufferName,
    {
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.viewport(0, 0, params.width, params.height);
        gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

        gl.clear_color(
            world.clear_color[0],
            world.clear_color[1],
            world.clear_color[2],
            1.0,
        );
        gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

        gl.use_program(&self.program);

        gl.active_texture(gl::TEXTURE0);
        gl.uniform_1i(&self.diffuse_sampler_loc, 0);

        // gl.active_texture(gl::TEXTURE1);
        // gl.bind_texture(gl::TEXTURE_2D, &self.normal_texture);
        // gl.uniform_1i(&self.normal_sampler_loc, 1);

        let pos_from_wld_to_clp = params.pos_from_cam_to_clp * world.camera.pos_from_wld_to_cam();

        gl.uniform_matrix4f(
            &self.pos_from_wld_to_clp_loc,
            gl::MajorAxis::Column,
            pos_from_wld_to_clp.as_ref(),
        );

        // Cache texture binding.
        let mut bound_diffuse_texture: u32 = 0;

        for i in 0..self.vaos.len() {
            if let Some(material_id) = world.models[i].mesh.material_id {
                let diffuse_texture = &self.diffuse_textures[material_id];
                if diffuse_texture.as_u32() != bound_diffuse_texture {
                    gl.bind_texture(gl::TEXTURE_2D, diffuse_texture);
                    bound_diffuse_texture = diffuse_texture.as_u32();
                }
            }

            let highlight: f32 = keyboard_model::Index::new(self.key_indices[i])
                .map(|i| world.keyboard_model.pressure(i))
                .unwrap_or(0.0);
            gl.uniform_1f(&self.highlight_loc, highlight);

            gl.bind_vertex_array(&self.vaos[i]);
            gl.draw_elements(gl::TRIANGLES, self.element_counts[i], gl::UNSIGNED_INT, 0);
        }

        gl.bind_vertex_array(&gl::Unbind);

        gl.bind_framebuffer(gl::FRAMEBUFFER, &gl::DefaultFramebufferName);
    }

    pub unsafe fn new(gl: &gl::Gl, world: &World) -> Self {
        let mut vs = gl
            .create_shader(gl::VERTEX_SHADER)
            .expect("Failed to create shader.");
        recompile_shader(&gl, &mut vs, VS_SRC).unwrap_or_else(|e| panic!("{}", e));

        let mut fs = gl
            .create_shader(gl::FRAGMENT_SHADER)
            .expect("Failed to create shader.");
        recompile_shader(&gl, &mut fs, FS_SRC).unwrap_or_else(|e| panic!("{}", e));

        let mut program = gl.create_program().expect("Failed to create program.");
        gl.attach_shader(&mut program, &vs);
        gl.attach_shader(&mut program, &fs);
        gl.link_program(&mut program);
        gl.use_program(&program);

        let vaos = {
            assert_eq!(
                mem::size_of::<Option<gl::VertexArrayName>>(),
                mem::size_of::<gl::VertexArrayName>()
            );
            // Create uninitialized memory.
            let mut names: Vec<Option<gl::VertexArrayName>> =
                Vec::with_capacity(world.models.len());
            names.set_len(world.models.len());
            gl.gen_vertex_arrays(&mut names);
            // Assert that all names != 0.
            for name in names.iter() {
                if let None = name {
                    panic!("Failed to acquire vertex array name.");
                }
            }
            let (ptr, len, cap) = (names.as_mut_ptr(), names.len(), names.capacity());
            mem::forget(names);
            Vec::from_raw_parts(ptr as *mut gl::VertexArrayName, len, cap)
        };

        let vbs = {
            assert_eq!(
                mem::size_of::<Option<gl::BufferName>>(),
                mem::size_of::<gl::BufferName>()
            );
            // Create uninitialized memory.
            let mut names: Vec<Option<gl::BufferName>> = Vec::with_capacity(world.models.len());
            names.set_len(world.models.len());
            gl.gen_buffers(&mut names);
            // Assert that all names != 0.
            for name in names.iter() {
                if let None = name {
                    panic!("Failed to acquire buffer name.");
                }
            }
            let (ptr, len, cap) = (names.as_mut_ptr(), names.len(), names.capacity());
            mem::forget(names);
            Vec::from_raw_parts(ptr as *mut gl::BufferName, len, cap)
        };

        let ebs = {
            assert_eq!(
                mem::size_of::<Option<gl::BufferName>>(),
                mem::size_of::<gl::BufferName>()
            );
            // Create uninitialized memory.
            let mut names: Vec<Option<gl::BufferName>> = Vec::with_capacity(world.models.len());
            names.set_len(world.models.len());
            gl.gen_buffers(&mut names);
            // Assert that all names != 0.
            for name in names.iter() {
                if let None = name {
                    panic!("Failed to acquire buffer name.");
                }
            }
            let (ptr, len, cap) = (names.as_mut_ptr(), names.len(), names.capacity());
            mem::forget(names);
            Vec::from_raw_parts(ptr as *mut gl::BufferName, len, cap)
        };

        let element_counts: Vec<usize> = world
            .models
            .iter()
            .map(|model| model.mesh.indices.len())
            .collect();

        let key_indices: Vec<keyboard_model::UncheckedIndex> = world
            .models
            .iter()
            .map(|model| model_name_to_keyboard_index(&model.name))
            .collect();

        for (i, model) in world.models.iter().enumerate() {
            let vao = &vaos[i];
            let vb = &vbs[i];
            let eb = &ebs[i];

            let ver_pos_size = mem::size_of_val(&model.mesh.positions[..]);
            let tex_pos_size = mem::size_of_val(&model.mesh.texcoords[..]);

            let ver_pos_offset = 0;
            let tex_pos_offset = ver_pos_size;

            gl.bind_vertex_array(vao);
            gl.bind_buffer(gl::ARRAY_BUFFER, vb);
            gl.buffer_reserve(
                gl::ARRAY_BUFFER,
                ver_pos_size + tex_pos_size,
                gl::STATIC_DRAW,
            );
            gl.buffer_sub_data(gl::ARRAY_BUFFER, ver_pos_offset, &model.mesh.positions[..]);
            gl.buffer_sub_data(gl::ARRAY_BUFFER, tex_pos_offset, &model.mesh.texcoords[..]);

            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, eb);
            gl.buffer_data(
                gl::ELEMENT_ARRAY_BUFFER,
                &model.mesh.indices,
                gl::STATIC_DRAW,
            );

            // AOS layout.

            let vs_ver_pos_loc = gl
                .get_attrib_location(&program, gl::static_cstr!("vs_ver_pos"))
                .expect("Could not find attribute location.");

            let vs_tex_pos_loc = gl
                .get_attrib_location(&program, gl::static_cstr!("vs_tex_pos"))
                .expect("Could not find attribute location.");

            gl.vertex_attrib_pointer(
                &vs_ver_pos_loc,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<[f32; 3]>(),
                ver_pos_offset,
            );

            gl.vertex_attrib_pointer(
                &vs_tex_pos_loc,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<[f32; 2]>(),
                tex_pos_offset,
            );

            gl.enable_vertex_attrib_array(&vs_ver_pos_loc);
            gl.enable_vertex_attrib_array(&vs_tex_pos_loc);

            gl.bind_vertex_array(&gl::Unbind);
            gl.bind_buffer(gl::ARRAY_BUFFER, &gl::Unbind);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, &gl::Unbind);
        }

        let pos_from_wld_to_clp_loc: gl::UniformLocation<[[f32; 4]; 4]> = gl
            .get_uniform_location(&program, gl::static_cstr!("pos_from_wld_to_clp"))
            .expect("Could not find uniform location.");

        let highlight_loc: gl::UniformLocation<f32> = gl
            .get_uniform_location(&program, gl::static_cstr!("highlight"))
            .expect("Could not find uniform location.");

        let diffuse_sampler_loc = gl
            .get_uniform_location(&program, gl::static_cstr!("diffuse_sampler"))
            .expect("Could not find attribute location.");

        // let normal_sampler_loc = gl
        //     .get_uniform_location(&program, gl::static_cstr!("normal_sampler"))
        //     .expect("Could not find attribute location.");


        let ebs = {
            assert_eq!(
                mem::size_of::<Option<gl::BufferName>>(),
                mem::size_of::<gl::BufferName>()
            );
            // Create uninitialized memory.
            let mut names: Vec<Option<gl::BufferName>> = Vec::with_capacity(world.models.len());
            names.set_len(world.models.len());
            gl.gen_buffers(&mut names);
            // Assert that all names != 0.
            for name in names.iter() {
                if let None = name {
                    panic!("Failed to acquire buffer name.");
                }
            }
            let (ptr, len, cap) = (names.as_mut_ptr(), names.len(), names.capacity());
            mem::forget(names);
            Vec::from_raw_parts(ptr as *mut gl::BufferName, len, cap)
        };

        let diffuse_textures = {
            assert_eq!(
                mem::size_of::<Option<gl::TextureName>>(),
                mem::size_of::<gl::TextureName>(),
            );
            let mut names: Vec<Option<gl::TextureName>> = Vec::with_capacity(world.materials.len());
            names.set_len(world.materials.len());
            gl.gen_textures(&mut names);
            // Assert that all names != 0.
            for name in names.iter() {
                if let None = name {
                    panic!("Failed to acquire texture name.");
                }
            }
            let (ptr, len, cap) = (names.as_mut_ptr(), names.len(), names.capacity());
            mem::forget(names);
            Vec::from_raw_parts(ptr as *mut gl::TextureName, len, cap)
        };

        for (i, material) in world.materials.iter().enumerate() {
            let path: std::path::PathBuf = ["data", material.diffuse_texture.as_ref()].iter().collect();

            {
                let img = image::open(path)
                    .unwrap()
                    .flipv()
                    .to_rgba();
                gl.bind_texture(gl::TEXTURE_2D, &diffuse_textures[i]);
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

        Renderer {
            program,
            pos_from_wld_to_clp_loc,
            highlight_loc,
            diffuse_sampler_loc,
            // normal_sampler_loc,
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
