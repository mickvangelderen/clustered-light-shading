use crate::rendering;
use crate::*;
use cgmath::*;
use gl_typed as gl;
use std::convert::TryFrom;

pub struct Renderer {
    pub program: rendering::Program,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
}

pub struct Parameters<'a> {
    pub vertices: &'a [[f32; 3]],
    pub indices: &'a [[u32; 2]],
    pub obj_to_clp: &'a Matrix4<f64>,
    pub color: [f32; 3],
}

glsl_defines!(fixed_header {
    bindings: {},
    uniforms: {
        OBJ_TO_CLP_LOC = 0;
        COLOR_LOC = 1;
    },
});

// TODO: Actually use this?
#[repr(C)]
struct Vertex {
    pub pos_in_obj: [f32; 3],
}

// We can draw vertex array data from 0..N (N is at least 16, can be queried) buffers.
const VERTEX_ARRAY_BUFFER_BINDING_INDEX: gl::VertexArrayBufferBindingIndex =
    gl::VertexArrayBufferBindingIndex::from_u32(0);

impl Renderer {
    pub fn render(&mut self, context: &mut RenderingContext, params: &Parameters) {
        unsafe {
            self.program.update(context);
            let gl = &context.gl;
            if let ProgramName::Linked(program_name) = self.program.name {
                gl.use_program(program_name);

                gl.uniform_matrix4f(
                    OBJ_TO_CLP_LOC,
                    gl::MajorAxis::Column,
                    params.obj_to_clp.cast().unwrap().as_ref(),
                );
                gl.uniform_3f(COLOR_LOC, params.color);

                gl.named_buffer_data(
                    self.vertex_buffer_name,
                    params.vertices.slice_as_bytes(),
                    gl::STREAM_DRAW,
                );
                gl.named_buffer_data(
                    self.element_buffer_name,
                    params.indices.slice_as_bytes(),
                    gl::STREAM_DRAW,
                );

                gl.bind_vertex_array(self.vertex_array_name);

                gl.enable(gl::DEPTH_TEST);
                gl.depth_func(gl::GREATER);
                gl.depth_mask(gl::WriteMask::Enabled);

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
    }

    pub fn new(context: &mut RenderingContext) -> Self {
        let gl = context.gl;
        unsafe {
            let vertex_array_name = gl.create_vertex_array();
            let vertex_buffer_name = gl.create_buffer();
            let element_buffer_name = gl.create_buffer();

            gl.vertex_array_attrib_format(
                vertex_array_name,
                rendering::VS_POS_IN_OBJ_LOC,
                resources::F32_3,
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
                program: vs_fs_program(context, "line_renderer.vert", "line_renderer.frag", fixed_header()),
                vertex_array_name,
                vertex_buffer_name,
                element_buffer_name,
            }
        }
    }
}
