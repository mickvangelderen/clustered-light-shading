use crate::parameters;
use crate::resources::Resources;
use crate::shader_defines;
use cgmath::*;
use gl_typed as gl;

unsafe fn recompile_shader(gl: &gl::Gl, name: gl::ShaderName, source: &[u8]) -> Result<(), String> {
    gl.shader_source(name, &[shader_defines::VERSION, shader_defines::DEFINES, source]);
    gl.compile_shader(name);
    let status = gl.get_shaderiv_move(name, gl::COMPILE_STATUS);
    if status == gl::ShaderCompileStatus::Compiled.into() {
        Ok(())
    } else {
        let log = gl.get_shader_info_log_move(name);
        Err(String::from_utf8(log).unwrap())
    }
}

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
    pub view_ind_uniforms: parameters::ViewIndependentUniforms,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub view_ind_params: parameters::ViewIndependentParameters,
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

            self.view_ind_uniforms.set(gl, params.view_ind_params);

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
                recompile_shader(gl, self.vertex_shader_name, bytes.as_ref()).unwrap_or_else(|e| eprintln!("{}", e));
                should_link = true;
            }

            if let Some(bytes) = update.fragment_shader {
                recompile_shader(gl, self.fragment_shader_name, bytes.as_ref()).unwrap_or_else(|e| eprintln!("{}", e));
                should_link = true;
            }

            if should_link {
                gl.link_program(self.program_name);
                gl.use_program(self.program_name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program_name, "pos_from_obj_to_wld");
                self.view_ind_uniforms.update(gl, self.program_name);

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
                view_ind_uniforms: Default::default(),
            }
        }
    }
}
