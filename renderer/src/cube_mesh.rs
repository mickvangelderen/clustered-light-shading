use crate::*;

#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    pos_in_obj: [f32; 3],
    nor_in_obj: [f32; 3],
    pos_in_tex: [f32; 2],
}

impl Vertex {
    pub unsafe fn set_format(gl: &gl::Gl, vao: gl::VertexArrayName, vb: gl::BufferName, eb: gl::BufferName) {
        let stride = std::mem::size_of::<Self>() as u32;

        let pos_in_obj_offset = 0u32;
        let nor_in_obj_offset = pos_in_obj_offset + std::mem::size_of::<[f32; 3]>() as u32;
        let pos_in_tex_offset = nor_in_obj_offset + std::mem::size_of::<[f32; 3]>() as u32;

        // Attribute layout specification.
        gl.vertex_array_attrib_format(
            vao,
            rendering::VS_POS_IN_OBJ_LOC,
            3,
            gl::FLOAT,
            false,
            pos_in_obj_offset,
        );
        gl.vertex_array_attrib_format(
            vao,
            rendering::VS_NOR_IN_OBJ_LOC,
            3,
            gl::FLOAT,
            false,
            nor_in_obj_offset,
        );
        gl.vertex_array_attrib_format(
            vao,
            rendering::VS_POS_IN_TEX_LOC,
            2,
            gl::FLOAT,
            false,
            pos_in_tex_offset,
        );

        gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_OBJ_LOC);
        gl.enable_vertex_array_attrib(vao, rendering::VS_NOR_IN_OBJ_LOC);
        gl.enable_vertex_array_attrib(vao, rendering::VS_POS_IN_TEX_LOC);

        // Attribute source specification.
        gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_OBJ_LOC, resources::BBI_00);
        gl.vertex_array_attrib_binding(vao, rendering::VS_NOR_IN_OBJ_LOC, resources::BBI_00);
        gl.vertex_array_attrib_binding(vao, rendering::VS_POS_IN_TEX_LOC, resources::BBI_00);

        gl.vertex_array_vertex_buffer(vao, resources::BBI_00, vb, 0, stride);

        // Element buffer.
        gl.vertex_array_element_buffer(vao, eb);
    }
}

pub type Triangle<T> = [T; 3];

pub fn generate(x: (f32, f32), y: (f32, f32), z: (f32, f32)) -> ([Vertex; 4 * 6], [Triangle<u32>; 2 * 6]) {
    let (x0, x1) = x;
    let (y0, y1) = y;
    let (z0, z1) = z;
    let (s0, s1) = (0.0, 1.0);
    let (t0, t1) = (0.0, 1.0);
    let nx = [-1.0, 0.0, 0.0];
    let px = [1.0, 0.0, 0.0];
    let ny = [0.0, -1.0, 0.0];
    let py = [0.0, 1.0, 0.0];
    let nz = [0.0, 0.0, -1.0];
    let pz = [0.0, 0.0, 1.0];
    (
        [
            // -X
            Vertex {
                pos_in_obj: [x0, y0, z0],
                pos_in_tex: [s0, t0],
                nor_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y0, z1],
                pos_in_tex: [s0, t1],
                nor_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y1, z1],
                pos_in_tex: [s1, t1],
                nor_in_obj: nx,
            },
            Vertex {
                pos_in_obj: [x0, y1, z0],
                pos_in_tex: [s1, t0],
                nor_in_obj: nx,
            },
            // -Y
            Vertex {
                pos_in_obj: [x0, y0, z0],
                pos_in_tex: [s0, t0],
                nor_in_obj: ny,
            },
            Vertex {
                pos_in_obj: [x1, y0, z0],
                pos_in_tex: [s0, t1],
                nor_in_obj: ny,
            },
            Vertex {
                pos_in_obj: [x1, y0, z1],
                pos_in_tex: [s1, t1],
                nor_in_obj: ny,
            },
            Vertex {
                pos_in_obj: [x0, y0, z1],
                pos_in_tex: [s1, t0],
                nor_in_obj: ny,
            },
            // -Z
            Vertex {
                pos_in_obj: [x0, y0, z0],
                pos_in_tex: [s0, t0],
                nor_in_obj: nz,
            },
            Vertex {
                pos_in_obj: [x0, y1, z0],
                pos_in_tex: [s0, t1],
                nor_in_obj: nz,
            },
            Vertex {
                pos_in_obj: [x1, y1, z0],
                pos_in_tex: [s1, t1],
                nor_in_obj: nz,
            },
            Vertex {
                pos_in_obj: [x1, y0, z0],
                pos_in_tex: [s1, t0],
                nor_in_obj: nz,
            },
            // +X
            Vertex {
                pos_in_obj: [x1, y1, z1],
                pos_in_tex: [s0, t0],
                nor_in_obj: px,
            },
            Vertex {
                pos_in_obj: [x1, y0, z1],
                pos_in_tex: [s0, t1],
                nor_in_obj: px,
            },
            Vertex {
                pos_in_obj: [x1, y0, z0],
                pos_in_tex: [s1, t1],
                nor_in_obj: px,
            },
            Vertex {
                pos_in_obj: [x1, y1, z0],
                pos_in_tex: [s1, t0],
                nor_in_obj: px,
            },
            // +Y
            Vertex {
                pos_in_obj: [x1, y1, z1],
                pos_in_tex: [s0, t0],
                nor_in_obj: py,
            },
            Vertex {
                pos_in_obj: [x1, y1, z0],
                pos_in_tex: [s0, t1],
                nor_in_obj: py,
            },
            Vertex {
                pos_in_obj: [x0, y1, z0],
                pos_in_tex: [s1, t1],
                nor_in_obj: py,
            },
            Vertex {
                pos_in_obj: [x0, y1, z1],
                pos_in_tex: [s1, t0],
                nor_in_obj: py,
            },
            // +Z
            Vertex {
                pos_in_obj: [x1, y1, z1],
                pos_in_tex: [s0, t0],
                nor_in_obj: pz,
            },
            Vertex {
                pos_in_obj: [x0, y1, z1],
                pos_in_tex: [s0, t1],
                nor_in_obj: pz,
            },
            Vertex {
                pos_in_obj: [x0, y0, z1],
                pos_in_tex: [s1, t1],
                nor_in_obj: pz,
            },
            Vertex {
                pos_in_obj: [x1, y0, z1],
                pos_in_tex: [s1, t0],
                nor_in_obj: pz,
            },
        ],
        [
            [0, 1, 2],
            [2, 3, 0],
            [4, 5, 6],
            [6, 7, 4],
            [8, 9, 10],
            [10, 11, 8],
            [12, 13, 14],
            [14, 15, 12],
            [16, 17, 18],
            [18, 19, 16],
            [20, 21, 22],
            [22, 23, 20],
        ],
    )
}
