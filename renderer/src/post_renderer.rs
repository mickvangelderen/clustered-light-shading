use crate::rendering;
use crate::resources::*;
use crate::World;
use crate::*;
use gl_typed as gl;
use std::convert::TryFrom;

pub struct Renderer {
    pub program: rendering::Program,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub nor_in_cam_sampler_loc: gl::OptionUniformLocation,
    pub ao_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_tex_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub viewport: Viewport<i32>,
    pub framebuffer: gl::FramebufferName,
    pub color_texture_name: gl::TextureName,
    pub depth_texture_name: gl::TextureName,
    pub nor_in_cam_texture_name: gl::TextureName,
    pub ao_texture_name: gl::TextureName,
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            params.viewport.set(gl);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

            self.update(gl, world);
            if let ProgramName::Linked(program_name) = self.program.name(&world.global) {
                gl.use_program(*program_name);

                if let Some(loc) = self.color_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.bind_texture_unit(0, params.color_texture_name);
                };

                if let Some(loc) = self.depth_sampler_loc.into() {
                    gl.uniform_1i(loc, 1);
                    gl.bind_texture_unit(1, params.depth_texture_name);
                };

                if let Some(loc) = self.nor_in_cam_sampler_loc.into() {
                    gl.uniform_1i(loc, 2);
                    gl.bind_texture_unit(2, params.nor_in_cam_texture_name);
                };

                if let Some(loc) = self.ao_sampler_loc.into() {
                    gl.uniform_1i(loc, 3);
                    gl.bind_texture_unit(3, params.ao_texture_name);
                };

                gl.bind_vertex_array(resources.full_screen_vao);
                gl.draw_elements(
                    gl::TRIANGLES,
                    u32::try_from(FULL_SCREEN_INDICES.len() * 3).unwrap(),
                    gl::UNSIGNED_INT,
                    0,
                );
                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) {
        let modified = self.program.modified();
        if modified < self.program.update(gl, world) {
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                unsafe {
                    self.color_sampler_loc = get_uniform_location!(gl, *name, "color_sampler");
                    self.depth_sampler_loc = get_uniform_location!(gl, *name, "depth_sampler");
                    self.nor_in_cam_sampler_loc = get_uniform_location!(gl, *name, "nor_in_cam_sampler");
                    self.ao_sampler_loc = get_uniform_location!(gl, *name, "ao_sampler");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: rendering::Program::new(
                gl,
                vec![world.add_source("post_renderer.vert")],
                vec![world.add_source("post_renderer.frag")],
            ),
            color_sampler_loc: gl::OptionUniformLocation::NONE,
            depth_sampler_loc: gl::OptionUniformLocation::NONE,
            nor_in_cam_sampler_loc: gl::OptionUniformLocation::NONE,
            ao_sampler_loc: gl::OptionUniformLocation::NONE,
            vs_pos_in_tex_loc: gl::OptionAttributeLocation::NONE,
        }
    }
}
