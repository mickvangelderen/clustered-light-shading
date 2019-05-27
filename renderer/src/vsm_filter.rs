use crate::rendering;
use std::convert::TryFrom;
use crate::resources::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub delta_loc: gl::OptionUniformLocation,
    pub sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_tex_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub width: i32,
    pub height: i32,
    pub framebuffer_x: gl::FramebufferName,
    pub framebuffer_xy: gl::FramebufferName,
    pub color: gl::TextureName,
    pub color_x: gl::TextureName,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.use_program(self.program.name);
            gl.bind_vertex_array(resources.full_screen_vao);

            // X pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer_x);
                gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

                if let Some(loc) = self.sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.active_texture(gl::TEXTURE0);
                    gl.bind_texture(gl::TEXTURE_2D, params.color);
                };

                if let Some(loc) = self.delta_loc.into() {
                    gl.uniform_2f(loc, [1.0 / params.width as f32, 0.0]);
                }

                gl.draw_elements(gl::TRIANGLES, u32::try_from(FULL_SCREEN_INDICES.len() * 3).unwrap(), gl::UNSIGNED_INT, 0);
            }

            // Y pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer_xy);
                gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

                if let Some(loc) = self.sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.active_texture(gl::TEXTURE0);
                    gl.bind_texture(gl::TEXTURE_2D, params.color_x);
                };

                if let Some(loc) = self.delta_loc.into() {
                    gl.uniform_2f(loc, [0.0, 1.0 / params.height as f32]);
                }

                gl.draw_elements(gl::TRIANGLES, u32::try_from(FULL_SCREEN_INDICES.len() * 3).unwrap(), gl::UNSIGNED_INT, 0);
            }

            gl.unbind_vertex_array();
            gl.unuse_program();

            gl.bind_texture(gl::TEXTURE_2D, params.color);
            gl.generate_mipmap(gl::TEXTURE_2D);
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);
                self.delta_loc = get_uniform_location!(gl, self.program.name, "delta");
                self.sampler_loc = get_uniform_location!(gl, self.program.name, "sampler");
                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        Renderer {
            program: rendering::VSFSProgram::new(gl),
            delta_loc: gl::OptionUniformLocation::NONE,
            sampler_loc: gl::OptionUniformLocation::NONE,
            vs_pos_in_tex_loc: gl::OptionAttributeLocation::NONE,
        }
    }
}
