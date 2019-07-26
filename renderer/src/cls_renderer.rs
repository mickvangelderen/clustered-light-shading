use crate::*;

pub struct Renderer {
    pub fragments_per_cluster_program: rendering::Program,
    pub compact_clusters_0_program: rendering::Program,
    pub compact_clusters_1_program: rendering::Program,
    pub compact_clusters_2_program: rendering::Program,
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

// compact clusters
pub const ITEM_COUNT_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };

pub const FRAGMENTS_PER_CLUSTER_BINDING: u32 = 0;
pub const OFFSET_BINDING: u32 = 1;
pub const ACTIVE_CLUSTER_BINDING: u32 = 2;
pub const DRAW_COMMAND_BINDING: u32 = 3;
pub const COMPUTE_COMMAND_BINDING: u32 = 4;
pub const LIGHT_BINDING: u32 = 5;
pub const LIGHT_COUNT_BINDING: u32 = 6;

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
        }
    }
}
