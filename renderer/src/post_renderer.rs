use crate::convert::*;
use crate::World;
use gl_typed as gl;
use gl_typed::convert::*;
use crate::frustrum::Frustrum;

static VERTICES: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

static INDICES: [[u32; 3]; 2] = [[0, 1, 2], [2, 3, 0]];

unsafe fn recompile_shader(gl: &gl::Gl, name: gl::ShaderName, source: &[u8]) -> Result<(), String> {
    gl.shader_source(name, &[source]);
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
    pub time_loc: gl::OptionUniformLocation,
    pub width_loc: gl::OptionUniformLocation,
    pub height_loc: gl::OptionUniformLocation,
    pub x0_loc: gl::OptionUniformLocation,
    pub x1_loc: gl::OptionUniformLocation,
    pub y0_loc: gl::OptionUniformLocation,
    pub y1_loc: gl::OptionUniformLocation,
    pub z0_loc: gl::OptionUniformLocation,
    pub z1_loc: gl::OptionUniformLocation,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub nor_in_cam_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_qua_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub color_texture_name: gl::TextureName,
    pub depth_texture_name: gl::TextureName,
    pub nor_in_cam_texture_name: gl::TextureName,
    pub frustrum: Frustrum,
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
    pub unsafe fn render(&self, gl: &gl::Gl, params: &Parameters, world: &World) {
        gl.disable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.viewport(0, 0, params.width, params.height);
        gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
        gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into()]);

        gl.use_program(self.program_name);

        if let Some(loc) = self.time_loc.into() {
            gl.uniform_1f(loc, world.time);
        }

        if let Some(loc) = self.width_loc.into() {
            gl.uniform_1i(loc, params.width);
        }

        if let Some(loc) = self.height_loc.into() {
            gl.uniform_1i(loc, params.height);
        }

        if let Some(loc) = self.x0_loc.into() {
            gl.uniform_1f(loc, params.frustrum.x0);
        }

        if let Some(loc) = self.x1_loc.into() {
            gl.uniform_1f(loc, params.frustrum.x1);
        }

        if let Some(loc) = self.y0_loc.into() {
            gl.uniform_1f(loc, params.frustrum.y0);
        }

        if let Some(loc) = self.y1_loc.into() {
            gl.uniform_1f(loc, params.frustrum.y1);
        }

        if let Some(loc) = self.z0_loc.into() {
            gl.uniform_1f(loc, params.frustrum.z0);
        }

        if let Some(loc) = self.z1_loc.into() {
            gl.uniform_1f(loc, params.frustrum.z1);
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

        gl.bind_vertex_array(self.vertex_array_name);

        gl.draw_elements(gl::TRIANGLES, INDICES.len() * 3, gl::UNSIGNED_INT, 0);

        gl.unbind_vertex_array();

        gl.bind_framebuffer(gl::FRAMEBUFFER, None);

        gl.unuse_program();
    }

    pub unsafe fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        let mut should_link = false;

        if let Some(bytes) = update.vertex_shader {
            recompile_shader(gl, self.vertex_shader_name, bytes.as_ref())
                .unwrap_or_else(|e| eprintln!("{}", e));
            should_link = true;
        }

        if let Some(bytes) = update.fragment_shader {
            recompile_shader(gl, self.fragment_shader_name, bytes.as_ref())
                .unwrap_or_else(|e| eprintln!("{}", e));
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

            self.time_loc = get_uniform_location!(gl, self.program_name, "time");
            self.width_loc = get_uniform_location!(gl, self.program_name, "width");
            self.height_loc = get_uniform_location!(gl, self.program_name, "height");
            self.x0_loc = get_uniform_location!(gl, self.program_name, "x0");
            self.x1_loc = get_uniform_location!(gl, self.program_name, "x1");
            self.y0_loc = get_uniform_location!(gl, self.program_name, "y0");
            self.y1_loc = get_uniform_location!(gl, self.program_name, "y1");
            self.z0_loc = get_uniform_location!(gl, self.program_name, "z0");
            self.z1_loc = get_uniform_location!(gl, self.program_name, "z1");
            self.color_sampler_loc = get_uniform_location!(gl, self.program_name, "color_sampler");
            self.depth_sampler_loc = get_uniform_location!(gl, self.program_name, "depth_sampler");
            self.nor_in_cam_sampler_loc = get_uniform_location!(gl, self.program_name, "nor_in_cam_sampler");

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

    pub unsafe fn new(gl: &gl::Gl) -> Self {
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

        let [vertex_buffer_name, element_buffer_name]: [gl::BufferName; 2] = {
            let mut names: [Option<gl::BufferName>; 2] = std::mem::uninitialized();
            gl.gen_buffers(&mut names);
            names.try_transmute_each().unwrap()
        };

        gl.bind_vertex_array(vertex_array_name);
        gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
        gl.buffer_data(gl::ARRAY_BUFFER, (&VERTICES[..]).flatten(), gl::STATIC_DRAW);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, (&INDICES[..]).flatten(), gl::STATIC_DRAW);
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
            time_loc: gl::OptionUniformLocation::NONE,
            width_loc: gl::OptionUniformLocation::NONE,
            height_loc: gl::OptionUniformLocation::NONE,
            x0_loc: gl::OptionUniformLocation::NONE,
            x1_loc: gl::OptionUniformLocation::NONE,
            y0_loc: gl::OptionUniformLocation::NONE,
            y1_loc: gl::OptionUniformLocation::NONE,
            z0_loc: gl::OptionUniformLocation::NONE,
            z1_loc: gl::OptionUniformLocation::NONE,
            color_sampler_loc: gl::OptionUniformLocation::NONE,
            depth_sampler_loc: gl::OptionUniformLocation::NONE,
            nor_in_cam_sampler_loc: gl::OptionUniformLocation::NONE,
            vs_pos_in_qua_loc: gl::OptionAttributeLocation::NONE,
        }
    }
}
