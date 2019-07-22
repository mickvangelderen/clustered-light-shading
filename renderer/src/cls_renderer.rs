use crate::*;

pub struct Renderer {
    pub fragments_per_cluster_program: rendering::Program,
    pub compact_clusters_0_program: rendering::Program,
    pub compact_clusters_1_program: rendering::Program,
    pub compact_clusters_2_program: rendering::Program,
    // pub fb_dims_loc: gl::OptionUniformLocation,
}

pub struct Buffer {
    name: gl::BufferName,
    byte_capacity: usize,
}

impl Buffer {
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            Self {
                name: gl.create_buffer(),
                byte_capacity: 0,
            }
        }
    }

    pub fn ensure_capacity<T>(&mut self, gl: &gl::Gl, capacity: usize) {
        unsafe {
            let byte_capacity = std::mem::size_of::<T>() * capacity;

            if self.byte_capacity < capacity {
                gl.named_buffer_reserve(self.name, byte_capacity, gl::DYNAMIC_DRAW);
                self.byte_capacity = byte_capacity;
            }
        }
    }
}

pub struct Resources {
    pub fragments_per_cluster_buffer: Buffer,
    pub offset_buffer: Buffer,
    pub active_cluster_buffer: Buffer,
}

impl Resources {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            fragments_per_cluster_buffer: Buffer::new(gl),
            offset_buffer: Buffer::new(gl),
            active_cluster_buffer: Buffer::new(gl),
        }
    }
}

pub struct RenderParams<'a> {
    pub gl: &'a gl::Gl,
    pub world: &'a mut World,
    pub cfg: &'a configuration::Root,
    pub resources: &'a mut Resources,
    pub depth_texture: gl::TextureName,
    pub depth_dims: Vector2<u32>,
    pub cluster_dims: Vector3<u32>,
    pub clp_to_cls: Matrix4<f32>,
}

// fragments per cluster program
pub const DEPTH_SAMPLER_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
pub const FB_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(1) };
pub const CLP_TO_CLS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(2) };
pub const CLUSTER_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(3) };

pub const FRAGMENTS_PER_CLUSTER_BINDING: u32 = 0;

// fn fragments_per_cluster_header() -> String {
//     format!(
//         "\n\
//          #define DEPTH_SAMPLER_LOC {}\n\
//          #define DEPTH_DIMS_LOC {}\n\
//          #define CLP_TO_CLS_LOC {}\n\
//          #define CLUSTER_DIMS_LOC {}\n\
//          #define FRAGMENTS_PER_CLUSTER_BINDING {}\n\
//          ",
//         DEPTH_SAMPLER_LOC.into_i32(),
//         DEPTH_DIMS_LOC.into_i32(),
//         CLP_TO_CLS_LOC.into_i32(),
//         CLUSTER_DIMS_LOC.into_i32(),
//         FRAGMENTS_PER_CLUSTER_BINDING,
//     )
// }

// // compact clusters program
// const ITEM_COUNT_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
// const ACTIVE_CLUSTER_CAPACITY: u32 = 1024 * 1024;

// fn compact_cluster_header(pass: u32) -> String {
//     format!(
//         r##"
// #define PASS {}
// #define ACTIVE_CLUSTER_CAPACITY {}
// #define ITEM_COUNT_LOC {}
// "##,
//         pass,
//         ACTIVE_CLUSTER_CAPACITY,
//         ITEM_COUNT_LOC.into_i32(),
//     )
// }

impl Renderer {
    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Renderer {
            fragments_per_cluster_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/fragments_per_cluster.comp"),
                )],
            ),
            compact_clusters_0_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_clusters_0.comp"),
                )],
            ),
            compact_clusters_1_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_clusters_1.comp"),
                )],
            ),
            compact_clusters_2_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_clusters_2.comp"),
                )],
            ),
            // dimensions_loc: gl::OptionUniformLocation::NONE,
            // text_sampler_loc: gl::OptionUniformLocation::NONE,
            // text_dimensions_loc: gl::OptionUniformLocation::NONE,
        }
    }
}
