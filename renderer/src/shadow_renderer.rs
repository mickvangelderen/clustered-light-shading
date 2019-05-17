use crate::gl_ext::*;
use crate::rendering;
use crate::resources::Resources;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
}

#[derive(Default)]
pub struct Update<B: AsRef<[u8]>> {
    pub vertex_shader: Option<B>,
    pub fragment_shader: Option<B>,
}

impl<B: AsRef<[u8]>> Update<B> {
    pub fn should_update(&self) -> bool {
        self.vertex_shader.is_some() || self.fragment_shader.is_some()
    }
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, resources: &Resources) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            // Reverse-Z.
            gl.clear_depth(0.0);
            gl.clear(gl::ClearFlags::COLOR_BUFFER_BIT | gl::ClearFlags::DEPTH_BUFFER_BIT);

            gl.use_program(self.program_name);

            for i in 0..resources.vaos.len() {
                if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                    let pos_from_obj_to_wld = Matrix4::from_translation(resources.meshes[i].translate);

                    gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_obj_to_wld.as_ref());
                }

                gl.bind_vertex_array(resources.vaos[i]);
                gl.draw_elements(gl::TRIANGLES, resources.element_counts[i], gl::UNSIGNED_INT, 0);
            }

            gl.unbind_vertex_array();

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
        }
    }

    pub fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        unsafe {
            let mut should_link = false;

            if let Some(bytes) = update.vertex_shader {
                self.vertex_shader_name
                    .compile(
                        gl,
                        &[
                            rendering::COMMON_DECLARATION.as_bytes(),
                            rendering::GLOBAL_DATA_DECLARATION.as_bytes(),
                            rendering::VIEW_DATA_DECLARATION.as_bytes(),
                            "#line 1 1\n".as_bytes(),
                            bytes.as_ref(),
                        ],
                    )
                    .unwrap_or_else(|e| eprintln!("{} (vertex):\n{}", file!(), e));
                should_link = true;
            }

            if let Some(bytes) = update.fragment_shader {
                self.fragment_shader_name
                    .compile(
                        gl,
                        &[
                            rendering::COMMON_DECLARATION.as_bytes(),
                            rendering::MATERIAL_DATA_DECLARATION.as_bytes(),
                            "#line 1 1\n".as_bytes(),
                            bytes.as_ref(),
                        ],
                    )
                    .unwrap_or_else(|e| eprintln!("{} (fragment):\n{}", file!(), e));
                should_link = true;
            }

            if should_link {
                self.program_name
                    .link(gl)
                    .unwrap_or_else(|e| eprintln!("{} (program):\n{}", file!(), e));

                gl.use_program(self.program_name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program_name, "pos_from_obj_to_wld");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let vertex_shader_name = gl.create_shader(gl::VERTEX_SHADER).expect("Failed to create shader.");

            let fragment_shader_name = gl.create_shader(gl::FRAGMENT_SHADER).expect("Failed to create shader.");

            let program_name = gl.create_program().expect("Failed to create program_name.");
            gl.attach_shader(program_name, vertex_shader_name);
            gl.attach_shader(program_name, fragment_shader_name);

            Renderer {
                program_name,
                vertex_shader_name,
                fragment_shader_name,
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
            }
        }
    }
}
