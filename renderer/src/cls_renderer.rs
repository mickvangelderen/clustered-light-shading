use crate::*;

pub struct Renderer {
    pub fragments_per_cluster_program: rendering::Program,
    pub compact_clusters_0_program: rendering::Program,
    pub compact_clusters_1_program: rendering::Program,
    pub compact_clusters_2_program: rendering::Program,
    pub compact_light_counts_0_program: rendering::Program,
    pub compact_light_counts_1_program: rendering::Program,
    pub compact_light_counts_2_program: rendering::Program,
}

// fragments per cluster program
pub const DEPTH_SAMPLER_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
pub const FB_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(1) };
pub const CLP_TO_CLS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(2) };
pub const CLUSTER_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(3) };

// compact clusters
pub const ITEM_COUNT_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };

pub const CLUSTER_FRAGMENT_COUNTS_BINDING: u32 = 0;
pub const ACTIVE_CLUSTER_INDICES_BINDING: u32 = 1;
pub const ACTIVE_CLUSTER_LIGHT_COUNTS_BINDING: u32 = 2;
pub const ACTIVE_CLUSTER_LIGHT_OFFSETS_BINDING: u32 = 3;
pub const LIGHT_XYZR_BINDING: u32 = 4;
pub const OFFSET_BINDING: u32 = 5;
pub const DRAW_COMMAND_BINDING: u32 = 6;
pub const COMPUTE_COMMAND_BINDING: u32 = 7;
pub const LIGHT_INDICES_BINDING: u32 = 8;

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
            compact_light_counts_0_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_light_counts_0.comp"),
                )],
            ),
            compact_light_counts_1_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_light_counts_1.comp"),
                )],
            ),
            compact_light_counts_2_program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/compact_light_counts_2.comp"),
                )],
            ),
        }
    }
}
