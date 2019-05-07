use crate::convert::*;
use crate::frustrum::Frustrum;
use crate::gl_ext::*;
use crate::keyboard_model;
use crate::resources::Resources;
use crate::shader_defines;
use crate::World;
use cgmath::*;
use gl_typed as gl;
use gl_typed::convert::*;

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
    pub pos_from_wld_to_cam_loc: gl::OptionUniformLocation,
    pub pos_from_wld_to_clp_loc: gl::OptionUniformLocation,
}

pub struct Parameters<'a> {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub pos_from_obj_to_wld: Matrix4<f32>,
    pub pos_from_cam_to_clp: Matrix4<f32>,
    pub vertices: &'a [[f32; 3]],
    pub indices: &'a [[u32; 2]],
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
    pub unsafe fn render<'a>(&self, gl: &gl::Gl, params: &Parameters<'a>, world: &World, resources: &Resources) {
        gl.enable(gl::DEPTH_TEST);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.viewport(0, 0, params.width, params.height);
        gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
        gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()]);

        gl.use_program(self.program_name);

        let pos_from_wld_to_cam = if world.smooth_camera {
            world.camera.smooth_pos_from_wld_to_cam()
        } else {
            world.camera.pos_from_wld_to_cam()
        };

        if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_obj_to_wld.as_ref());
        }

        if let Some(loc) = self.pos_from_wld_to_cam_loc.into() {
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_wld_to_cam.as_ref());
        }

        if let Some(loc) = self.pos_from_wld_to_clp_loc.into() {
            let pos_from_wld_to_clp = params.pos_from_cam_to_clp * pos_from_wld_to_cam;
            gl.uniform_matrix4f(loc, gl::MajorAxis::Column, pos_from_wld_to_clp.as_ref());
        }

        gl.bind_vertex_array(self.vertex_array_name);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.vertex_buffer_name);
        gl.buffer_data(gl::ARRAY_BUFFER, params.vertices.flatten(), gl::STREAM_DRAW);
        // gl.vertex_attrib_pointer(
        //     shader_defines::VS_POS_IN_OBJ_LOC,
        //     3,
        //     gl::FLOAT,
        //     gl::FALSE,
        //     std::mem::size_of::<[f32; 3]>(),
        //     0,
        // );
        // gl.enable_vertex_attrib_array(shader_defines::VS_POS_IN_OBJ_LOC);

        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer_name);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, params.indices.flatten(), gl::STATIC_DRAW);

        gl.draw_elements(gl::LINES, params.indices.flatten().len(), gl::UNSIGNED_INT, 0);

        gl.unbind_vertex_array();

        gl.bind_framebuffer(gl::FRAMEBUFFER, None);
        gl.unuse_program();
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

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program_name, "pos_from_obj_to_wld");
                self.pos_from_wld_to_cam_loc = get_uniform_location!(gl, self.program_name, "pos_from_wld_to_cam");
                self.pos_from_wld_to_clp_loc = get_uniform_location!(gl, self.program_name, "pos_from_wld_to_clp");

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

            let [vertex_buffer_name, element_buffer_name]: [gl::BufferName; 2] = {
                let mut names: [Option<gl::BufferName>; 2] = std::mem::uninitialized();
                gl.gen_buffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_vertex_array(vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
            gl.buffer_reserve(gl::ARRAY_BUFFER, 4, gl::STREAM_DRAW);
            gl.vertex_attrib_pointer(
                shader_defines::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<[f32; 3]>(),
                0,
            );
            gl.enable_vertex_attrib_array(shader_defines::VS_POS_IN_OBJ_LOC);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
            gl.buffer_reserve(gl::ELEMENT_ARRAY_BUFFER, 4, gl::STATIC_DRAW);
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
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
                pos_from_wld_to_cam_loc: gl::OptionUniformLocation::NONE,
                pos_from_wld_to_clp_loc: gl::OptionUniformLocation::NONE,
            }
        }
    }
}
