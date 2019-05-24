use crate::rendering;
use crate::resources::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub default_colors_loc: gl::OptionUniformLocation,
    pub color_matrix_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub color_texture_name: gl::TextureName,
    pub default_colors: [f32; 4],
    pub color_matrix: [[f32; 4]; 4],
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, resources: &Resources) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.depth_mask(gl::WriteMask::Disabled);
            gl.viewport(params.x0, params.y0, params.x1 - params.x0, params.y1 - params.y0);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

            gl.use_program(self.program.name);

            if let Some(loc) = self.color_sampler_loc.into() {
                gl.uniform_1i(loc, 0);
                gl.active_texture(gl::TEXTURE0);
                gl.bind_texture(gl::TEXTURE_2D, params.color_texture_name);
            };

            if let Some(loc) = self.default_colors_loc.into() {
                gl.uniform_4f(loc, params.default_colors);
            };

            if let Some(loc) = self.color_matrix_loc.into() {
                gl.uniform_matrix4f(loc, gl::MajorAxis::Row, &params.color_matrix);
            };

            gl.bind_vertex_array(resources.full_screen_vao);
            gl.draw_elements(gl::TRIANGLES, FULL_SCREEN_INDICES.len() * 4, gl::UNSIGNED_INT, 0);
            gl.unbind_vertex_array();
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
            gl.depth_mask(gl::WriteMask::Enabled);
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);

                macro_rules! get_uniform_location {
                    ($gl: ident, $program: expr, $s: expr) => {{
                        let loc = $gl.get_uniform_location($program, gl::static_cstr!($s));
                        if loc.is_none() {
                            eprintln!("{}: Could not get uniform location {:?}.", file!(), $s);
                        }
                        loc
                    }};
                }

                self.color_sampler_loc = get_uniform_location!(gl, self.program.name, "color_sampler");
                self.default_colors_loc = get_uniform_location!(gl, self.program.name, "default_colors");
                self.color_matrix_loc = get_uniform_location!(gl, self.program.name, "color_matrix");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        Renderer {
            program: rendering::VSFSProgram::new(gl),
            color_sampler_loc: gl::OptionUniformLocation::NONE,
            default_colors_loc: gl::OptionUniformLocation::NONE,
            color_matrix_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
