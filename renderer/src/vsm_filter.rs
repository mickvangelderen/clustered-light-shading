use crate::convert::*;
use gl_typed as gl;
use gl_typed::convert::*;
use crate::shader_defines;

static VERTICES: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

static INDICES: [[u32; 3]; 2] = [[0, 1, 2], [2, 3, 0]];


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
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub delta_loc: gl::OptionUniformLocation,
    pub sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_qua_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub width: i32,
    pub height: i32,
    pub framebuffer_x: gl::FramebufferName,
    pub framebuffer_xy: gl::FramebufferName,
    pub color: gl::TextureName,
    pub color_x: gl::TextureName,
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
            gl.viewport(0, 0, params.width, params.height);
            gl.use_program(self.program_name);
            gl.bind_vertex_array(self.vertex_array_name);

            // X pass.
            {
                gl.bind_framebuffer(gl::FRAMEBUFFER, Some(params.framebuffer_x));
                gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

                if let Some(loc) = self.sampler_loc.into() {
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

                if let Some(loc) = self.sampler_loc.into() {
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

            gl.bind_texture(gl::TEXTURE_2D, params.color);
            gl.generate_mipmap(gl::TEXTURE_2D);
        }
    }

    pub fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        unsafe {
            let mut should_link = false;

            if let Some(bytes) = update.vertex_shader {
                recompile_shader(gl, self.vertex_shader_name, bytes.as_ref())
                    .unwrap_or_else(|e| eprintln!("{} (vertex):\n{}", file!(), e));
                should_link = true;
            }

            if let Some(bytes) = update.fragment_shader {
                recompile_shader(gl, self.fragment_shader_name, bytes.as_ref())
                    .unwrap_or_else(|e| eprintln!("{} (fragment):\n{}", file!(), e));
                should_link = true;
            }

            if should_link {
                gl.link_program(self.program_name);
                gl.use_program(self.program_name);

                macro_rules! get_uniform_location {
                    ($gl: ident, $program: expr, $s: expr) => {{
                        let loc = $gl.get_uniform_location($program, gl::static_cstr!($s));
                        if loc.is_none() {
                            eprintln!("{}: Could not get uniform location {:?}.", file!(), $s);
                        }
                        loc
                    }};
                }

                self.delta_loc = get_uniform_location!(gl, self.program_name, "delta");
                self.sampler_loc = get_uniform_location!(gl, self.program_name, "sampler");

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

                self.vs_pos_in_qua_loc =
                    get_attribute_location!(gl, self.program_name, "vs_pos_in_qua");

                // Set up attributes.

                gl.bind_buffer(gl::ARRAY_BUFFER, self.vertex_buffer_name);
                gl.bind_vertex_array(self.vertex_array_name);

                if let Some(loc) = self.vs_pos_in_qua_loc.into() {
                    gl.vertex_attrib_pointer(
                        loc,
                        2,
                        gl::FLOAT,
                        gl::FALSE,
                        std::mem::size_of::<[f32; 2]>(),
                        0,
                    );

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
            let vertex_shader_name = gl
                .create_shader(gl::VERTEX_SHADER)
                .expect("Failed to create shader.");

            let fragment_shader_name = gl
                .create_shader(gl::FRAGMENT_SHADER)
                .expect("Failed to create shader.");

            let program_name = gl.create_program().expect("Failed to create program_name.");
            gl.attach_shader(program_name, vertex_shader_name);
            gl.attach_shader(program_name, fragment_shader_name);

            let [vertex_array_name]: [gl::VertexArrayName; 1] = {
                let mut names: [Option<gl::VertexArrayName>; 1] = std::mem::uninitialized();
                gl.gen_vertex_arrays(&mut names);
                names.try_transmute_each().unwrap()
            };

            let [vertex_buffer_name, element_buffer_name]: [gl::BufferName;
                2] = {
                let mut names: [Option<gl::BufferName>; 2] = std::mem::uninitialized();
                gl.gen_buffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_vertex_array(vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
            gl.buffer_data(gl::ARRAY_BUFFER, (&VERTICES[..]).flatten(), gl::STATIC_DRAW);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
            gl.buffer_data(
                gl::ELEMENT_ARRAY_BUFFER,
                (&INDICES[..]).flatten(),
                gl::STATIC_DRAW,
            );
            gl.unbind_vertex_array();
            gl.unbind_buffer(gl::ARRAY_BUFFER);
            gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);

            Renderer {
                program_name,
                vertex_shader_name,
                fragment_shader_name,
                vertex_array_name,
                vertex_buffer_name,
                element_buffer_name,
                delta_loc: gl::OptionUniformLocation::NONE,
                sampler_loc: gl::OptionUniformLocation::NONE,
                vs_pos_in_qua_loc: gl::OptionAttributeLocation::NONE,
            }
        }
    }
}
