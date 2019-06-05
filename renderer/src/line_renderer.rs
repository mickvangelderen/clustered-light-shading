use crate::*;
use crate::convert::*;
use crate::rendering;
use cgmath::*;
use gl_typed as gl;
use std::convert::TryFrom;

pub struct Renderer {
    pub program: rendering::VSFSProgram,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
}

pub struct Parameters<'a> {
    pub viewport: Viewport<i32>,
    pub framebuffer: gl::FramebufferName,
    pub vertices: &'a [[f32; 3]],
    pub indices: &'a [[u32; 2]],
    pub pos_from_obj_to_wld: &'a Matrix4<f32>,
}

// TODO: Actually use this?
#[repr(C)]
struct Vertex {
    pub pos_in_obj: [f32; 3],
}

// We can draw vertex array data from 0..N (N is at least 16, can be queried) buffers.
const VERTEX_ARRAY_BUFFER_BINDING_INDEX: gl::VertexArrayBufferBindingIndex =
    gl::VertexArrayBufferBindingIndex::from_u32(0);

impl Renderer {
    pub fn render<'a>(&self, gl: &gl::Gl, params: &Parameters<'a>) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            params.viewport.set(gl);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);

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
            gl.draw_elements(
                gl::LINES,
                u32::try_from(params.indices.flatten().len()).unwrap(),
                gl::UNSIGNED_INT,
                0,
            );
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

            gl.vertex_array_attrib_format(
                vertex_array_name,
                rendering::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                false,
                field_offset!(Vertex, pos_in_obj) as u32,
            );
            gl.enable_vertex_array_attrib(vertex_array_name, rendering::VS_POS_IN_OBJ_LOC);
            gl.vertex_array_attrib_binding(
                vertex_array_name,
                rendering::VS_POS_IN_OBJ_LOC,
                VERTEX_ARRAY_BUFFER_BINDING_INDEX,
            );

            gl.vertex_array_vertex_buffer(
                vertex_array_name,
                VERTEX_ARRAY_BUFFER_BINDING_INDEX,
                vertex_buffer_name,
                0,
                std::mem::size_of::<Vertex>() as u32,
            );
            gl.vertex_array_element_buffer(vertex_array_name, element_buffer_name);

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
