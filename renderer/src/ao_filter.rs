use crate::*;
use crate::rendering;
use crate::resources::*;
use std::convert::TryFrom;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub delta_loc: gl::OptionUniformLocation,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_tex_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub viewport: Viewport<i32>,
    pub framebuffer_x: gl::FramebufferName,
    pub framebuffer_xy: gl::FramebufferName,
    pub color: gl::TextureName,
    pub color_x: gl::TextureName,
    pub depth: gl::TextureName,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            params.viewport.set(gl);
            gl.use_program(self.program.name);
            gl.bind_vertex_array(resources.full_screen_vao);

            if let Some(loc) = self.depth_sampler_loc.into() {
                gl.uniform_1i(loc, 1);
                gl.bind_texture_unit(1, params.depth);
            };

            // X pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer_x);

                if let Some(loc) = self.color_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.bind_texture_unit(0, params.color);
                };

                if let Some(loc) = self.delta_loc.into() {
                    gl.uniform_2f(loc, [1.0 / params.viewport.dimensions.x as f32, 0.0]);
                }

                gl.draw_elements(gl::TRIANGLES, u32::try_from(FULL_SCREEN_INDICES.len() * 3).unwrap(), gl::UNSIGNED_INT, 0);
            }

            // Y pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer_xy);

                if let Some(loc) = self.color_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.bind_texture_unit(0, params.color_x);
                };

                if let Some(loc) = self.delta_loc.into() {
                    gl.uniform_2f(loc, [0.0, 1.0 / params.viewport.dimensions.y as f32]);
                }

                gl.draw_elements(gl::TRIANGLES, u32::try_from(FULL_SCREEN_INDICES.len() * 3).unwrap(), gl::UNSIGNED_INT, 0);
            }

            gl.unbind_vertex_array();
            gl.unuse_program();
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);
                self.delta_loc = get_uniform_location!(gl, self.program.name, "delta");
                self.color_sampler_loc = get_uniform_location!(gl, self.program.name, "color_sampler");
                self.depth_sampler_loc = get_uniform_location!(gl, self.program.name, "depth_sampler");
                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        Renderer {
            program: rendering::VSFSProgram::new(gl),
            delta_loc: gl::OptionUniformLocation::NONE,
            color_sampler_loc: gl::OptionUniformLocation::NONE,
            depth_sampler_loc: gl::OptionUniformLocation::NONE,
            vs_pos_in_tex_loc: gl::OptionAttributeLocation::NONE,
        }
    }
}
