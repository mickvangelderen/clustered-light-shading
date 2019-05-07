use crate::convert::*;
use crate::gl_ext::*;
use crate::shader_defines;
use gl_typed as gl;
use gl_typed::convert::*;

static VERTICES: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
static INDICES: [[u32; 4]; 1] = [[0, 1, 2, 3]];

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub color_sampler_loc: gl::OptionUniformLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub color_texture_name: gl::TextureName,
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

            gl.use_program(self.program_name);

            if let Some(loc) = self.color_sampler_loc.into() {
                gl.uniform_1i(loc, 0);
                gl.active_texture(gl::TEXTURE0);
                gl.bind_texture(gl::TEXTURE_2D, params.color_texture_name);
            };

            gl.bind_vertex_array(self.vertex_array_name);
            gl.draw_elements(gl::TRIANGLE_STRIP, INDICES.len() * 4, gl::UNSIGNED_INT, 0);
            gl.unbind_vertex_array();
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
            gl.depth_mask(gl::WriteMask::Enabled);
        }
    }

    pub fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        unsafe {
            let mut should_link = false;

            if let Some(bytes) = update.vertex_shader {
                self.vertex_shader_name
                    .compile(gl, &[shader_defines::VERSION, shader_defines::DEFINES, bytes.as_ref()])
                    .unwrap_or_else(|e| eprintln!("{} (vertex):\n{}", file!(), e));
                should_link = true;
            }

            if let Some(bytes) = update.fragment_shader {
                self.fragment_shader_name
                    .compile(gl, &[shader_defines::VERSION, shader_defines::DEFINES, bytes.as_ref()])
                    .unwrap_or_else(|e| eprintln!("{} (fragment):\n{}", file!(), e));
                should_link = true;
            }

            if should_link {
                self.program_name
                    .link(gl)
                    .unwrap_or_else(|e| eprintln!("{} (program):\n{}", file!(), e));

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

                self.color_sampler_loc = get_uniform_location!(gl, self.program_name, "color_sampler");

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let vertex_shader_name = gl.create_shader(gl::VERTEX_SHADER).expect("Failed to create shader.");

            let fragment_shader_name = gl.create_shader(gl::FRAGMENT_SHADER).expect("Failed to create shader.");

            let program_name = gl.create_program().expect("Failed to create program_name.");
            gl.attach_shader(program_name, vertex_shader_name);
            gl.attach_shader(program_name, fragment_shader_name);

            let [vertex_array_name]: [gl::VertexArrayName; 1] = {
                let mut names: [Option<gl::VertexArrayName>; 1] = std::mem::uninitialized();
                gl.gen_vertex_arrays(&mut names);
                names.try_transmute_each().unwrap()
            };

            let [vertex_buffer_name, element_buffer_name, hbao_kernel_buffer_name]: [gl::BufferName; 3] = {
                let mut names: [Option<gl::BufferName>; 3] = std::mem::uninitialized();
                gl.gen_buffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_buffer(gl::UNIFORM_BUFFER, hbao_kernel_buffer_name);
            gl.buffer_data(
                gl::UNIFORM_BUFFER,
                crate::random_unit_sphere_volume::get(),
                gl::STATIC_DRAW,
            );
            gl.unbind_buffer(gl::UNIFORM_BUFFER);
            const HBAO_KERNEL_BUFFER_BINDING: u32 = 0;
            gl.bind_buffer_base(gl::UNIFORM_BUFFER, HBAO_KERNEL_BUFFER_BINDING, hbao_kernel_buffer_name);

            gl.bind_vertex_array(vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
            gl.buffer_data(gl::ARRAY_BUFFER, (&VERTICES[..]).flatten(), gl::STATIC_DRAW);
            gl.vertex_attrib_pointer(
                shader_defines::VS_POS_IN_TEX_LOC,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<[f32; 2]>(),
                0,
            );
            gl.enable_vertex_attrib_array(shader_defines::VS_POS_IN_TEX_LOC);
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
                color_sampler_loc: gl::OptionUniformLocation::NONE,
            }
        }
    }
}
