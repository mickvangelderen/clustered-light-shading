use crate::parameters;
use crate::convert::*;
use crate::gl_ext::*;
use crate::shader_defines;
use crate::World;
use gl_typed as gl;
use gl_typed::convert::*;

static VERTICES: [[f32; 2]; 3] = [[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]];
static INDICES: [[u32; 3]; 1] = [[0, 1, 2]];

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
    pub view_dep_uniforms: parameters::ViewDependentUniforms,
    pub color_sampler_loc: gl::OptionUniformLocation,
    pub depth_sampler_loc: gl::OptionUniformLocation,
    pub nor_in_cam_sampler_loc: gl::OptionUniformLocation,
    pub random_unit_sphere_surface_sampler_loc: gl::OptionUniformLocation,
    pub vs_pos_in_qua_loc: gl::OptionAttributeLocation,
}

pub struct Parameters {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub color_texture_name: gl::TextureName,
    pub depth_texture_name: gl::TextureName,
    pub nor_in_cam_texture_name: gl::TextureName,
    pub random_unit_sphere_surface_texture_name: gl::TextureName,
    pub view_dep_params: parameters::ViewDependentParameters,
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
    pub fn render(&self, gl: &gl::Gl, params: &Parameters, world: &World) {
        unsafe {
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

            self.view_dep_uniforms.set(gl, params.view_dep_params);

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

            if let Some(loc) = self.random_unit_sphere_surface_sampler_loc.into() {
                gl.uniform_1i(loc, 3);
                gl.active_texture(gl::TEXTURE3);
                gl.bind_texture(gl::TEXTURE_2D, params.random_unit_sphere_surface_texture_name);
            };

            gl.bind_vertex_array(self.vertex_array_name);
            // // NOTE: Help renderdoc
            // gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer_name);

            gl.draw_elements(gl::TRIANGLES, INDICES.len() * 3, gl::UNSIGNED_INT, 0);

            gl.unbind_vertex_array();

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);

            gl.unuse_program();
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

                self.time_loc = get_uniform_location!(gl, self.program_name, "time");
                self.width_loc = get_uniform_location!(gl, self.program_name, "width");
                self.height_loc = get_uniform_location!(gl, self.program_name, "height");
                self.view_dep_uniforms.update(gl, self.program_name);
                self.color_sampler_loc = get_uniform_location!(gl, self.program_name, "color_sampler");
                self.depth_sampler_loc = get_uniform_location!(gl, self.program_name, "depth_sampler");
                self.nor_in_cam_sampler_loc = get_uniform_location!(gl, self.program_name, "nor_in_cam_sampler");
                self.random_unit_sphere_surface_sampler_loc =
                    get_uniform_location!(gl, self.program_name, "random_unit_sphere_surface_sampler");

                // Disable old locations.
                gl.bind_vertex_array(self.vertex_array_name);

                if let Some(loc) = self.vs_pos_in_qua_loc.into() {
                    gl.disable_vertex_attrib_array(loc);
                }

                gl.unbind_vertex_array();

                // Obtain new locations.
                self.vs_pos_in_qua_loc = get_attribute_location!(gl, self.program_name, "vs_pos_in_qua");

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

            let [vertex_buffer_name, element_buffer_name, ao_sample_buffer_name]: [gl::BufferName; 3] = {
                let mut names: [Option<gl::BufferName>; 3] = std::mem::uninitialized();
                gl.gen_buffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_buffer(gl::SHADER_STORAGE_BUFFER, ao_sample_buffer_name);
            gl.buffer_data(
                gl::SHADER_STORAGE_BUFFER,
                crate::random_unit_sphere_dense::get(),
                gl::STATIC_DRAW,
            );
            gl.bind_buffer_base(
                gl::SHADER_STORAGE_BUFFER,
                shader_defines::AO_SAMPLE_BUFFER_BINDING,
                ao_sample_buffer_name,
            );
            gl.unbind_buffer(gl::SHADER_STORAGE_BUFFER);

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
                view_dep_uniforms: Default::default(),
                color_sampler_loc: gl::OptionUniformLocation::NONE,
                depth_sampler_loc: gl::OptionUniformLocation::NONE,
                nor_in_cam_sampler_loc: gl::OptionUniformLocation::NONE,
                random_unit_sphere_surface_sampler_loc: gl::OptionUniformLocation::NONE,
                vs_pos_in_qua_loc: gl::OptionAttributeLocation::NONE,
            }
        }
    }
}
