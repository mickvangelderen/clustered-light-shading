use crate::convert::*;
use crate::gl_ext::*;
use crate::rendering;
use gl_typed as gl;
use gl_typed::convert::*;

static VERTICES: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
static INDICES: [[u32; 4]; 1] = [[0, 1, 2, 3]];

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
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
    pub fn render(&self, gl: &gl::Gl, params: &Parameters) {
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

            gl.bind_vertex_array(self.vertex_array_name);
            gl.draw_elements(gl::TRIANGLE_STRIP, INDICES.len() * 4, gl::UNSIGNED_INT, 0);
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
        unsafe {
            let [vertex_array_name]: [gl::VertexArrayName; 1] = {
                let mut names: [Option<gl::VertexArrayName>; 1] = std::mem::uninitialized();
                gl.gen_vertex_arrays(&mut names);
                names.try_transmute_each().unwrap()
            };

            let [vertex_buffer_name, element_buffer_name]: [gl::BufferName; 2] = {
                let mut names: [Option<gl::BufferName>; 2] = std::mem::uninitialized();
                gl.gen_buffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_vertex_array(vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
            gl.buffer_data(gl::ARRAY_BUFFER, (&VERTICES[..]).flatten(), gl::STATIC_DRAW);
            gl.vertex_attrib_pointer(
                rendering::VS_POS_IN_TEX_LOC,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<[f32; 2]>(),
                0,
            );
            gl.enable_vertex_attrib_array(rendering::VS_POS_IN_TEX_LOC);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
            gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, (&INDICES[..]).flatten(), gl::STATIC_DRAW);
            gl.unbind_vertex_array();
            gl.unbind_buffer(gl::ARRAY_BUFFER);
            gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);

            Renderer {
                program: rendering::VSFSProgram::new(gl),
                vertex_array_name,
                vertex_buffer_name,
                element_buffer_name,
                color_sampler_loc: gl::OptionUniformLocation::NONE,
                default_colors_loc: gl::OptionUniformLocation::NONE,
                color_matrix_loc: gl::OptionUniformLocation::NONE,
            }
        }
    }
}
