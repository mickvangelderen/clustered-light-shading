use crate::*;
use crate::convert::*;
use crate::rendering;
use crate::resources::*;
use crate::World;
use gl_typed as gl;
use std::convert::TryFrom;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub ao_sample_buffer_name: gl::BufferName,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub nor_in_cam_sampler_loc: gl::OptionUniformLocation,
    pub random_unit_sphere_surface_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_tex_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub viewport: Viewport<i32>,
    pub framebuffer: gl::FramebufferName,
    pub color_texture_name: gl::TextureName,
    pub depth_texture_name: gl::TextureName,
    pub nor_in_cam_texture_name: gl::TextureName,
    pub random_unit_sphere_surface_texture_name: gl::TextureName,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, _world: &World, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            params.viewport.set(gl);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

            gl.use_program(self.program.name);

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

            if let Some(loc) = self.random_unit_sphere_surface_sampler_loc.into() {
                gl.uniform_1i(loc, 3);
                gl.bind_texture_unit(3, params.random_unit_sphere_surface_texture_name);
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

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);

                self.color_sampler_loc = get_uniform_location!(gl, self.program.name, "color_sampler");
                self.depth_sampler_loc = get_uniform_location!(gl, self.program.name, "depth_sampler");
                self.nor_in_cam_sampler_loc = get_uniform_location!(gl, self.program.name, "nor_in_cam_sampler");
                self.random_unit_sphere_surface_sampler_loc =
                    get_uniform_location!(gl, self.program.name, "random_unit_sphere_surface_sampler");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let ao_sample_buffer_name = gl.create_buffer();

            gl.named_buffer_data(
                ao_sample_buffer_name,
                crate::random_unit_sphere_dense::get().slice_to_bytes(),
                gl::STATIC_DRAW,
            );
            gl.bind_buffer_base(
                gl::SHADER_STORAGE_BUFFER,
                rendering::AO_SAMPLE_BUFFER_BINDING,
                ao_sample_buffer_name,
            );

            Renderer {
                program: rendering::VSFSProgram::new(gl),
                ao_sample_buffer_name,
                color_sampler_loc: gl::OptionUniformLocation::NONE,
                depth_sampler_loc: gl::OptionUniformLocation::NONE,
                nor_in_cam_sampler_loc: gl::OptionUniformLocation::NONE,
                random_unit_sphere_surface_sampler_loc: gl::OptionUniformLocation::NONE,
                vs_pos_in_tex_loc: gl::OptionAttributeLocation::NONE,
            }
        }
    }
}
