use crate::convert::*;
use crate::rendering;
use gl_typed as gl;

static VERTICES: [[f32; 2]; 3] = [[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]];
static INDICES: [[u32; 3]; 1] = [[0, 1, 2]];

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub delta_loc: gl::OptionUniformLocation,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_qua_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub width: i32,
    pub height: i32,
    pub framebuffer_x: gl::FramebufferName,
    pub framebuffer_xy: gl::FramebufferName,
    pub color: gl::TextureName,
    pub color_x: gl::TextureName,
    pub depth: gl::TextureName,
}

impl Renderer {
    pub fn render(&self, gl: &gl::Gl, params: &Parameters) {
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.use_program(self.program.name);
            gl.bind_vertex_array(self.vertex_array_name);

            if let Some(loc) = self.depth_sampler_loc.into() {
                gl.uniform_1i(loc, 1);
                gl.active_texture(gl::TEXTURE1);
                gl.bind_texture(gl::TEXTURE_2D, params.depth);
            };

            // X pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, Some(params.framebuffer_x));
                gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

                if let Some(loc) = self.color_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.active_texture(gl::TEXTURE0);
                    gl.bind_texture(gl::TEXTURE_2D, params.color);
                };

                if let Some(loc) = self.delta_loc.into() {
                    gl.uniform_2f(loc, [1.0 / params.width as f32, 0.0]);
                }

                gl.draw_elements(gl::TRIANGLES, INDICES.len() * 3, gl::UNSIGNED_INT, 0);
            }

            // Y pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, Some(params.framebuffer_xy));
                gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

                if let Some(loc) = self.color_sampler_loc.into() {
                    gl.uniform_1i(loc, 0);
                    gl.active_texture(gl::TEXTURE0);
                    gl.bind_texture(gl::TEXTURE_2D, params.color_x);
                };

                if let Some(loc) = self.delta_loc.into() {
                    gl.uniform_2f(loc, [0.0, 1.0 / params.height as f32]);
                }

                gl.draw_elements(gl::TRIANGLES, INDICES.len() * 3, gl::UNSIGNED_INT, 0);
            }

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unbind_vertex_array();
            gl.unuse_program();
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

                self.delta_loc = get_uniform_location!(gl, self.program.name, "delta");
                self.color_sampler_loc = get_uniform_location!(gl, self.program.name, "color_sampler");
                self.depth_sampler_loc = get_uniform_location!(gl, self.program.name, "depth_sampler");

                // Disable old locations.
                gl.bind_vertex_array(self.vertex_array_name);

                if let Some(loc) = self.vs_pos_in_qua_loc.into() {
                    gl.disable_vertex_attrib_array(loc);
                }

                gl.unbind_vertex_array();

                // Obtain new locations.
                macro_rules! get_attribute_location {
                    ($gl: ident, $program: expr, $s: expr) => {{
                        let loc = $gl.get_attrib_location($program, gl::static_cstr!($s));
                        if loc.is_none() {
                            eprintln!("{}: Could not get attribute location {:?}.", file!(), $s);
                        }
                        loc
                    }};
                }

                self.vs_pos_in_qua_loc = get_attribute_location!(gl, self.program.name, "vs_pos_in_qua");

                // Set up attributes.

                gl.bind_buffer(gl::ARRAY_BUFFER, self.vertex_buffer_name);
                gl.bind_vertex_array(self.vertex_array_name);

                if let Some(loc) = self.vs_pos_in_qua_loc.into() {
                    gl.vertex_attrib_pointer(loc, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<[f32; 2]>(), 0);

                    gl.enable_vertex_attrib_array(loc);
                }

                gl.unbind_vertex_array();
                gl.unbind_buffer(gl::ARRAY_BUFFER);

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let vertex_array_name = gl.create_vertex_array();
            let vertex_buffer_name = gl.create_buffer();
            let element_buffer_name = gl.create_buffer();

            gl.bind_vertex_array(vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
            gl.named_buffer_data(vertex_buffer_name, VERTICES.slice_to_bytes(), gl::STATIC_DRAW);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
            gl.named_buffer_data(element_buffer_name, INDICES.slice_to_bytes(), gl::STATIC_DRAW);
            gl.unbind_vertex_array();
            gl.unbind_buffer(gl::ARRAY_BUFFER);
            gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);

            Renderer {
                program: rendering::VSFSProgram::new(gl),
                vertex_array_name,
                vertex_buffer_name,
                element_buffer_name,
                delta_loc: gl::OptionUniformLocation::NONE,
                color_sampler_loc: gl::OptionUniformLocation::NONE,
                depth_sampler_loc: gl::OptionUniformLocation::NONE,
                vs_pos_in_qua_loc: gl::OptionAttributeLocation::NONE,
            }
        }
    }
}
