use crate::rendering;
use crate::resources::*;
use crate::World;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub width_loc: gl::OptionUniformLocation,
    pub height_loc: gl::OptionUniformLocation,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub nor_in_cam_sampler_loc: gl::OptionUniformLocation,
    pub ao_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_tex_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub color_texture_name: gl::TextureName,
    pub depth_texture_name: gl::TextureName,
    pub nor_in_cam_texture_name: gl::TextureName,
    pub ao_texture_name: gl::TextureName,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, _world: &World, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);
            gl.use_program(self.program.name);

            if let Some(loc) = self.width_loc.into() {
                gl.uniform_1i(loc, params.width);
            }

            if let Some(loc) = self.height_loc.into() {
                gl.uniform_1i(loc, params.height);
            }

            if let Some(loc) = self.color_sampler_loc.into() {
                gl.uniform_1i(loc, 0);
                gl.active_texture(gl::TEXTURE0);
                gl.bind_texture(gl::TEXTURE_2D, params.color_texture_name);
            };

            if let Some(loc) = self.depth_sampler_loc.into() {
                gl.uniform_1i(loc, 1);
                gl.active_texture(gl::TEXTURE1);
                gl.bind_texture(gl::TEXTURE_2D, params.depth_texture_name);
            };

            if let Some(loc) = self.nor_in_cam_sampler_loc.into() {
                gl.uniform_1i(loc, 2);
                gl.active_texture(gl::TEXTURE2);
                gl.bind_texture(gl::TEXTURE_2D, params.nor_in_cam_texture_name);
            };

            if let Some(loc) = self.ao_sampler_loc.into() {
                gl.uniform_1i(loc, 3);
                gl.active_texture(gl::TEXTURE3);
                gl.bind_texture(gl::TEXTURE_2D, params.ao_texture_name);
            };

            gl.bind_vertex_array(resources.full_screen_vao);
            gl.draw_elements(gl::TRIANGLES, FULL_SCREEN_INDICES.len() * 3, gl::UNSIGNED_INT, 0);
            gl.unbind_vertex_array();
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);
                self.width_loc = get_uniform_location!(gl, self.program.name, "width");
                self.height_loc = get_uniform_location!(gl, self.program.name, "height");
                self.color_sampler_loc = get_uniform_location!(gl, self.program.name, "color_sampler");
                self.depth_sampler_loc = get_uniform_location!(gl, self.program.name, "depth_sampler");
                self.nor_in_cam_sampler_loc = get_uniform_location!(gl, self.program.name, "nor_in_cam_sampler");
                self.ao_sampler_loc = get_uniform_location!(gl, self.program.name, "ao_sampler");
                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        Renderer {
            program: rendering::VSFSProgram::new(gl),
            width_loc: gl::OptionUniformLocation::NONE,
            height_loc: gl::OptionUniformLocation::NONE,
            color_sampler_loc: gl::OptionUniformLocation::NONE,
            depth_sampler_loc: gl::OptionUniformLocation::NONE,
            nor_in_cam_sampler_loc: gl::OptionUniformLocation::NONE,
            ao_sampler_loc: gl::OptionUniformLocation::NONE,
            vs_pos_in_tex_loc: gl::OptionAttributeLocation::NONE,
        }
    }
}
