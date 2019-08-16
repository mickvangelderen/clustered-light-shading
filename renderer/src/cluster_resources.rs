use crate::*;

impl_enum_and_enum_map! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
    enum ClusterStage => struct ClusterStages {
        CompactClusters => compact_clusters,
        UploadLights => upload_lights,
        CountLights => count_lights,
        LightOffsets => light_offsets,
        AssignLights => assign_lights,
    }
}

impl ClusterStage {
    pub const VALUES: [ClusterStage; 5] = [
        ClusterStage::CompactClusters,
        ClusterStage::UploadLights,
        ClusterStage::CountLights,
        ClusterStage::LightOffsets,
        ClusterStage::AssignLights,
    ];

    pub fn title(self) -> &'static str {
        match self {
            ClusterStage::CompactClusters => "compact clusters",
            ClusterStage::UploadLights => "upload lights",
            ClusterStage::CountLights => "count lights",
            ClusterStage::LightOffsets => "comp light offs",
            ClusterStage::AssignLights => "assign lights",
        }
    }
}


pub struct ClusterResources {
    // GPU
    pub cluster_fragment_counts_buffer: DynamicBuffer,
    pub active_cluster_indices_buffer: DynamicBuffer,
    pub active_cluster_light_counts_buffer: DynamicBuffer,
    pub active_cluster_light_offsets_buffer: DynamicBuffer,
    pub light_xyzr_buffer: DynamicBuffer,
    pub offset_buffer: DynamicBuffer,
    pub draw_command_buffer: DynamicBuffer,
    pub compute_commands_buffer: DynamicBuffer,
    pub light_indices_buffer: DynamicBuffer,
    pub profilers: ClusterStages<Profiler>,
    // CPU
    pub active_clusters: Vec<u32>,
    pub active_cluster_lengths: Vec<u32>,
    pub active_cluster_offsets: Vec<u32>,
    pub light_indices: Vec<u32>,
    // Other
    pub camera_resources_pool: ClusterCameraResourcesPool,
    pub parameters: ClusterParameters,
}

impl ClusterResources {
    pub fn new(gl: &gl::Gl, cfg: &configuration::ClusteredLightShading) -> Self {
        Self {
            cluster_fragment_counts_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "cluster_fragment_counts");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_clusters as usize);
                buffer
            },
            active_cluster_indices_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "active_cluster_indices");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_active_clusters as usize);
                buffer
            },
            active_cluster_light_counts_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "active_cluster_light_counts");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_active_clusters as usize);
                buffer
            },
            active_cluster_light_offsets_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "active_cluster_light_offsets");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_active_clusters as usize);
                buffer
            },
            light_xyzr_buffer: unsafe {
                let buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "light_xyrz");
                buffer
            },
            offset_buffer: unsafe {
                let buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "offsets");
                buffer
            },
            draw_command_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "draw_comands");
                let data = DrawCommand {
                    count: 36,
                    prim_count: 0,
                    first_index: 0,
                    base_vertex: 0,
                    base_instance: 0,
                };
                buffer.ensure_capacity(gl, data.value_as_bytes().len());
                buffer.write(gl, data.value_as_bytes());
                buffer
            },
            compute_commands_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "compute_commands");
                let data = vec![
                    ComputeCommand {
                        work_group_x: 0,
                        work_group_y: 1,
                        work_group_z: 1,
                    },
                    ComputeCommand {
                        work_group_x: 0,
                        work_group_y: 1,
                        work_group_z: 1,
                    },
                ];
                buffer.ensure_capacity(gl, data.vec_as_bytes().len());
                buffer.write(gl, data.vec_as_bytes());
                buffer
            },
            light_indices_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "light_indices");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_light_indices as usize);
                buffer
            },
            profilers: ClusterStages::new(|_| Profiler::new(gl)),

            active_clusters: Vec::new(),
            active_cluster_lengths: Vec::new(),
            active_cluster_offsets: Vec::new(),
            light_indices: Vec::new(),

            camera_resources_pool: ClusterCameraResourcesPool::new(),
            parameters: Default::default(),
        }
    }

    pub fn reset(&mut self, _gl: &gl::Gl, _cfg: &configuration::ClusteredLightShading) {
        self.camera_resources_pool.reset();
        if cfg!(debug_assertions) {
            self.parameters = Default::default();
        }
    }
}

impl_frame_pool! {
    ClusterResourcesPool,
    ClusterResources,
    ClusterResourcesIndex,
    ClusterResourcesIndexIter,
    (gl: &gl::Gl, cfg: &configuration::ClusteredLightShading),
}
