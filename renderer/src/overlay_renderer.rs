use crate::*;
use crate::resources;

pub struct Renderer {
    pub program: rendering::Program,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub default_colors_loc: gl::OptionUniformLocation,
    pub color_matrix_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub framebuffer: gl::FramebufferName,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub color_texture_name: gl::TextureName,
    pub default_colors: [f32; 4],
    pub color_matrix: [[f32; 4]; 4],
}

impl Renderer {
    pub fn render(&mut self, gl: &gl::Gl, params: &Parameters, world: &mut World, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.depth_mask(gl::WriteMask::Disabled);
            gl.viewport(params.x0, params.y0, params.x1 - params.x0, params.y1 - params.y0);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

            self.update(gl, world);
            if let ProgramName::Linked(program_name) = self.program.name(&world.global) {
                gl.use_program(*program_name);

                if let Some(loc) = self.color_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.bind_texture_unit(0, params.color_texture_name);
                };

                if let Some(loc) = self.default_colors_loc.into() {
                    gl.uniform_4f(loc, params.default_colors);
                };

                if let Some(loc) = self.color_matrix_loc.into() {
                    gl.uniform_matrix4f(loc, gl::MajorAxis::Row, &params.color_matrix);
                };

                gl.bind_vertex_array(resources.full_screen_vao);
                gl.draw_elements(
                    gl::TRIANGLES,
                    u32::try_from(resources::FULL_SCREEN_INDICES.len() * 3).unwrap(),
                    gl::UNSIGNED_INT,
                    0,
                );
                gl.unbind_vertex_array();
                gl.unuse_program();
            }
            gl.depth_mask(gl::WriteMask::Enabled);
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) {
        let modified = self.program.modified();
        if modified < self.program.update(gl, world) {
            if let ProgramName::Linked(name) = self.program.name(&world.global) {
                unsafe {
                    self.color_sampler_loc = get_uniform_location!(gl, *name, "color_sampler");
                    self.default_colors_loc = get_uniform_location!(gl, *name, "default_colors");
                    self.color_matrix_loc = get_uniform_location!(gl, *name, "color_matrix");
                }
            }
        }
    }

    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            program: rendering::Program::new(
                gl,
                vec![world.add_source("overlay_renderer.vert")],
                vec![world.add_source("overlay_renderer.frag")],
            ),
            color_sampler_loc: gl::OptionUniformLocation::NONE,
            default_colors_loc: gl::OptionUniformLocation::NONE,
            color_matrix_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
