use crate::convert::*;
use crate::rendering;
use cgmath::*;
use gl_typed as gl;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters<'a> {
    pub framebuffer: gl::FramebufferName,
    pub width: i32,
    pub height: i32,
    pub vertices: &'a [[f32; 3]],
    pub indices: &'a [[u32; 2]],
    pub pos_from_obj_to_wld: &'a Matrix4<f32>,
}

impl Renderer {
    pub fn render<'a>(&self, gl: &gl::Gl, params: &Parameters<'a>) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()]);

            gl.use_program(self.program.name);

            if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.pos_from_obj_to_wld.as_ref());
            }

            gl.named_buffer_data(
                self.vertex_buffer_name,
                params.vertices.slice_to_bytes(),
                gl::STREAM_DRAW,
            );
            gl.named_buffer_data(
                self.element_buffer_name,
                params.indices.slice_to_bytes(),
                gl::STREAM_DRAW,
            );

            gl.bind_vertex_array(self.vertex_array_name);
            gl.draw_elements(gl::LINES, params.indices.flatten().len(), gl::UNSIGNED_INT, 0);
            gl.unbind_vertex_array();

            gl.unuse_program();
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, update: &rendering::VSFSProgramUpdate) {
        unsafe {
            if self.program.update(gl, update) {
                gl.use_program(self.program.name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program.name, "pos_from_obj_to_wld");
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
            gl.named_buffer_reserve(vertex_buffer_name, 4, gl::STREAM_DRAW);
            gl.vertex_attrib_pointer(
                rendering::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                false,
                std::mem::size_of::<[f32; 3]>(),
                0,
            );
            gl.enable_vertex_attrib_array(rendering::VS_POS_IN_OBJ_LOC);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
            gl.named_buffer_reserve(element_buffer_name, 4, gl::STATIC_DRAW);
            gl.unbind_vertex_array();
            gl.unbind_buffer(gl::ARRAY_BUFFER);
            gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);

            Renderer {
                program: rendering::VSFSProgram::new(gl),
                vertex_array_name,
                vertex_buffer_name,
                element_buffer_name,
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
            }
        }
    }
}
