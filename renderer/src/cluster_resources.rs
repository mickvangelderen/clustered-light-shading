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

pub struct ClusterParameters {
    pub configuration: configuration::ClusteredLightShading,
    pub wld_to_ccam: Matrix4<f64>,
    pub ccam_to_wld: Matrix4<f64>,
}

#[derive(Debug)]
pub struct ClusterComputed {
    pub dimensions: Vector3<u32>,
    pub frustum: Frustum<f64>, // useful for finding perspective transform frustum planes for intersection tests in shaders.
    pub ccam_to_cclp: Matrix4<f64>,
    pub cclp_to_ccam: Matrix4<f64>,
}

impl std::default::Default for ClusterComputed {
    fn default() -> Self {
        Self {
            dimensions: Vector3::zero(),
            frustum: Frustum::<f64>::zero(),
            ccam_to_cclp: Matrix4::identity(),
            cclp_to_ccam: Matrix4::identity(),
        }
    }
}

impl ClusterComputed {
    pub fn cluster_count(&self) -> u32 {
        self.dimensions.product()
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
    pub computed: ClusterComputed,
}

impl ClusterResources {
    pub fn new(gl: &gl::Gl, parameters: ClusterParameters) -> Self {
        let cfg = &parameters.configuration;
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
            parameters,
            computed: Default::default(),
        }
    }

    pub fn recompute(&mut self) {
        let parameters = &self.parameters;
        let cfg = &parameters.configuration;

        // Compute bounding box of all camera frustum corners.
        let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
        let range = Range3::<f64>::from_points({
            self.camera_resources_pool.used_slice().iter().flat_map(
                |&ClusterCameraResources {
                     parameters: ref cam_par,
                     ..
                 }| {
                    let clp_to_ccam = cam_par.clp_to_cam * cam_par.cam_to_wld * parameters.wld_to_ccam;
                    corners_in_clp
                        .into_iter()
                        .map(move |&p| clp_to_ccam.transform_point(p).cast::<f64>().unwrap())
                },
            )
        })
        .unwrap();

        let delta = range.delta();
        let dimensions = (delta / cfg.cluster_side as f64).map(f64::ceil);
        let dimensions_u32 = dimensions.cast::<u32>().unwrap();

        if dimensions_u32.product() > cfg.max_clusters {
            panic!(
                "Cluster dimensions are too large: {} x {} x {} exceeds the maximum {}.",
                dimensions_u32.x, dimensions_u32.y, dimensions_u32.z, cfg.max_clusters,
            );
        }

        let (frustum, ccam_to_cclp) = match cfg.projection {
            configuration::ClusteringProjection::Orthogonal => {
                let frustum = Frustum::<f64>::from_range(&range);
                let ccam_to_cclp = frustum.cluster_orthogonal(&Range3::from_vector(dimensions));
                (frustum, ccam_to_cclp)
            }
            configuration::ClusteringProjection::Perspective => {
                let cameras = self.camera_resources_pool.used_slice();
                assert_eq!(1, cameras.len());
                let frustum = cameras[0].parameters.frustum;
                let ccam_to_cclp = frustum.cluster_perspective(&Range3::from_vector(dimensions));
                (frustum, ccam_to_cclp)
            }
        };

        let cclp_to_ccam = ccam_to_cclp.invert().unwrap();

        self.computed = ClusterComputed {
            dimensions: dimensions.cast().unwrap(),
            ccam_to_cclp,
            cclp_to_ccam,
            frustum,
        };
    }

    pub fn reset(&mut self, _gl: &gl::Gl, parameters: ClusterParameters) {
        // TODO: Resize buffers?
        self.camera_resources_pool.reset();
        self.parameters = parameters;
        if cfg!(debug_assertions) {
            self.computed = Default::default();
        }
    }
}

impl_frame_pool! {
    ClusterResourcesPool,
    ClusterResources,
    ClusterResourcesIndex,
    ClusterResourcesIndexIter,
    (gl: &gl::Gl, parameters: ClusterParameters),
}
