use renderer::*;
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
            ClusterStage::CompactClusters => "cluster.compact_clusters",
            ClusterStage::UploadLights => "cluster.upload_lights",
            ClusterStage::CountLights => "cluster.count_lights",
            ClusterStage::LightOffsets => "cluster.compact_lights",
            ClusterStage::AssignLights => "cluster.assign_lights",
        }
    }
}

#[derive(Debug)]
pub struct ClusterParameters {
    pub configuration: ClusteredLightShadingConfiguration,
    pub wld_to_hmd: Matrix4<f64>,
    pub hmd_to_wld: Matrix4<f64>,
}

#[derive(Debug)]
pub struct ClusterComputed {
    pub dimensions: Vector3<u32>,
    pub frustum: Frustum<f64>, // useful for finding perspective transform frustum planes for intersection tests in shaders.
    pub wld_to_ccam: Matrix4<f64>,
    pub ccam_to_wld: Matrix4<f64>,
}

impl std::default::Default for ClusterComputed {
    fn default() -> Self {
        Self {
            dimensions: Vector3::zero(),
            frustum: Frustum::<f64>::zero(),
            wld_to_ccam: Matrix4::identity(),
            ccam_to_wld: Matrix4::identity(),
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
    pub cluster_space_buffer: DynamicBuffer,
    pub cluster_fragment_counts_buffer: DynamicBuffer,
    pub active_cluster_indices_buffer: DynamicBuffer,
    pub active_cluster_light_counts_buffer: DynamicBuffer,
    pub active_cluster_light_offsets_buffer: DynamicBuffer,
    pub light_xyzr_buffer: DynamicBuffer,
    pub offset_buffer: DynamicBuffer,
    pub draw_commands_buffer: DynamicBuffer,
    pub compute_commands_buffer: DynamicBuffer,
    pub light_indices_buffer: DynamicBuffer,
    pub profiling_cluster_buffer: DynamicBuffer,
    pub profilers: ClusterStages<SampleIndex>,
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
    pub fn new(gl: &gl::Gl, profiling_context: &mut ProfilingContext, parameters: ClusterParameters) -> Self {
        let cfg = &parameters.configuration;
        Self {
            cluster_space_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "cluster_space");
                buffer.ensure_capacity(gl, std::mem::size_of::<ClusterSpaceBuffer>());
                buffer
            },
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
            draw_commands_buffer: unsafe {
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
            profiling_cluster_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "profiling_cluster_buffer");
                buffer.ensure_capacity(gl, std::mem::size_of::<profiling::ClusterBuffer>());
                buffer
            },
            profilers: ClusterStages::new(|stage| profiling_context.add_sample(stage.title())),

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

        // TODO: Refactor, this is for multiple orthographic cameras but turns
        // out to work for a single perspective camera.

        // Compute bounding box of all camera frustum corners.
        let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
        let range = Range3::<f64>::from_points({
            self.camera_resources_pool.used_slice().iter().flat_map(
                |&ClusterCameraResources {
                     parameters: ref cam_par,
                     ..
                 }| {
                    let clp_to_hmd = parameters.wld_to_hmd * cam_par.cam_to_wld * cam_par.clp_to_cam;
                    corners_in_clp
                        .into_iter()
                        .map(move |&p| clp_to_hmd.transform_point(p).cast::<f64>().unwrap())
                },
            )
        })
        .unwrap();

        let dimensions = range.delta().div_element_wise(cfg.cluster_sides()).map(f64::ceil);

        let dimensions_u32 = dimensions.cast::<u32>().unwrap();

        if dimensions_u32.product() > cfg.max_clusters {
            panic!(
                "Cluster dimensions are too large: {} x {} x {} exceeds the maximum {}.",
                dimensions_u32.x, dimensions_u32.y, dimensions_u32.z, cfg.max_clusters,
            );
        }

        struct Things {
            frustum: Frustum<f64>,
            wld_to_ccam: Matrix4<f64>,
            ccam_to_wld: Matrix4<f64>,
        }

        let things: Things = match cfg.projection {
            ClusteringProjection::Orthographic => Things {
                frustum: Frustum::<f64>::from_range(&range),
                wld_to_ccam: parameters.wld_to_hmd,
                ccam_to_wld: parameters.hmd_to_wld,
            },
            ClusteringProjection::Perspective => {
                let cameras = self.camera_resources_pool.used_slice();

                match cameras.len() {
                    1 => Things {
                        frustum: cameras[0].parameters.frustum,
                        wld_to_ccam: cameras[0].parameters.wld_to_cam,
                        ccam_to_wld: cameras[0].parameters.cam_to_wld,
                    },
                    2 => {
                        let far_pos_in_clp = [
                            Point3::new(-1.0, -1.0, DEPTH_RANGE.1),
                            Point3::new(-1.0, 1.0, DEPTH_RANGE.1),
                            Point3::new(1.0, -1.0, DEPTH_RANGE.1),
                            Point3::new(1.0, 1.0, DEPTH_RANGE.1),
                        ];

                        let near_pos_in_clp = [
                            Point3::new(-1.0, -1.0, DEPTH_RANGE.0),
                            Point3::new(-1.0, 1.0, DEPTH_RANGE.0),
                            Point3::new(1.0, -1.0, DEPTH_RANGE.0),
                            Point3::new(1.0, 1.0, DEPTH_RANGE.0),
                        ];

                        let far_pos_in_hmd: Vec<Point3<f64>> = self
                            .camera_resources_pool
                            .used_slice()
                            .iter()
                            .flat_map(|camera| {
                                let clp_to_hmd: Matrix4<f64> =
                                    parameters.wld_to_hmd * camera.parameters.cam_to_wld * camera.parameters.clp_to_cam;
                                far_pos_in_clp
                                    .iter()
                                    .map(move |&pos_in_clp| clp_to_hmd.transform_point(pos_in_clp))
                            })
                            .collect();

                        let near_pos_in_hmd: Vec<Point3<f64>> = self
                            .camera_resources_pool
                            .used_slice()
                            .iter()
                            .flat_map(|camera| {
                                let clp_to_hmd: Matrix4<f64> =
                                    parameters.wld_to_hmd * camera.parameters.cam_to_wld * camera.parameters.clp_to_cam;
                                near_pos_in_clp
                                    .iter()
                                    .map(move |&pos_in_clp| clp_to_hmd.transform_point(pos_in_clp))
                            })
                            .collect();

                        fn take_xz(v: Vector3<f64>) -> Vector2<f64> {
                            Vector2::new(v.x, v.z)
                        }

                        fn take_yz(v: Vector3<f64>) -> Vector2<f64> {
                            Vector2::new(v.y, v.z)
                        }

                        #[derive(Debug)]
                        struct Plane {
                            fi: usize,
                            ni: usize,
                            z: f64,
                        }

                        let mut nx_max: Option<Plane> = None;
                        let mut px_max: Option<Plane> = None;
                        for (fi, &f) in far_pos_in_hmd.iter().enumerate() {
                            for (ni, &n) in near_pos_in_hmd.iter().enumerate() {
                                // Find intersection with z.
                                let dx = n.x - f.x;
                                if dx.abs() < std::f64::EPSILON {
                                    // No intersection.
                                    continue;
                                } else {
                                    // Test where all points lie.
                                    let f_to_n = take_xz(n - f);
                                    let mut all_pos = true;
                                    let mut all_neg = true;
                                    for &p in near_pos_in_hmd.iter().chain(far_pos_in_hmd.iter()) {
                                        let f_to_p = take_xz(p - f);
                                        let sign = f_to_n.perp_dot(f_to_p);
                                        if sign < 0.0 {
                                            all_pos = false;
                                        }
                                        if sign > 0.0 {
                                            all_neg = false;
                                        }
                                    }

                                    let z = (f.z * n.x - f.x * n.z) / dx;

                                    if all_pos {
                                        if match nx_max {
                                            Some(ref plane) => z > plane.z,
                                            None => true,
                                        } {
                                            nx_max = Some(Plane { fi, ni, z })
                                        }
                                    }

                                    if all_neg {
                                        if match px_max {
                                            Some(ref plane) => z > plane.z,
                                            None => true,
                                        } {
                                            px_max = Some(Plane { fi, ni, z })
                                        }
                                    }
                                }
                            }
                        }

                        let mut ny_max: Option<Plane> = None;
                        let mut py_max: Option<Plane> = None;
                        for (fi, &f) in far_pos_in_hmd.iter().enumerate() {
                            for (ni, &n) in near_pos_in_hmd.iter().enumerate() {
                                // Find intersection with z.
                                let dy = n.y - f.y;

                                if dy.abs() < std::f64::EPSILON {
                                    // No intersection.
                                    continue;
                                }

                                let z = (f.z * n.y - f.y * n.z) / dy;

                                if z < n.z {
                                    // Intersection not on the right side of the z axis.
                                    continue;
                                }

                                // Test where all points lie.
                                let f_to_n = take_yz(n - f);
                                let mut all_pos = true;
                                let mut all_neg = true;
                                for &p in far_pos_in_hmd.iter().chain(near_pos_in_hmd.iter()) {
                                    let f_to_p = take_yz(p - f);
                                    let sign = f_to_n.perp_dot(f_to_p);
                                    if sign < 0.0 {
                                        all_pos = false;
                                    }
                                    if sign > 0.0 {
                                        all_neg = false;
                                    }
                                }

                                if all_pos {
                                    if match ny_max {
                                        Some(ref plane) => z > plane.z,
                                        None => true,
                                    } {
                                        ny_max = Some(Plane { fi, ni, z })
                                    }
                                }

                                if all_neg {
                                    if match py_max {
                                        Some(ref plane) => z > plane.z,
                                        None => true,
                                    } {
                                        py_max = Some(Plane { fi, ni, z })
                                    }
                                }
                            }
                        }

                        let nx_max = nx_max.unwrap();
                        let px_max = px_max.unwrap();
                        let ny_max = ny_max.unwrap();
                        let py_max = py_max.unwrap();

                        let planes = [nx_max, px_max, ny_max, py_max];

                        let p_max = planes.iter().max_by(|a, b| a.z.partial_cmp(&b.z).unwrap()).unwrap();

                        let mut x0 = None;
                        let mut x1 = None;
                        let mut y0 = None;
                        let mut y1 = None;
                        let mut z0 = None;
                        let mut z1 = None;

                        let origin = Point3::new(0.0, 0.0, p_max.z);
                        for &p in far_pos_in_hmd.iter().chain(near_pos_in_hmd.iter()) {
                            if match z0 {
                                Some(z0) => p.z < z0,
                                None => true,
                            } {
                                z0 = Some(p.z);
                            }

                            if match z1 {
                                Some(z1) => p.z > z1,
                                None => true,
                            } {
                                z1 = Some(p.z)
                            }

                            let o_to_p = p - origin;
                            let mut all_nx = true;
                            let mut all_px = true;
                            let mut all_ny = true;
                            let mut all_py = true;
                            for &q in far_pos_in_hmd.iter().chain(near_pos_in_hmd.iter()) {
                                let o_to_q = q - origin;
                                let sign_x = take_xz(o_to_p).perp_dot(take_xz(o_to_q));
                                if sign_x > 0.0 {
                                    all_nx = false;
                                }
                                if sign_x < 0.0 {
                                    all_px = false;
                                }
                                let sign_y = take_yz(o_to_p).perp_dot(take_yz(o_to_q));
                                if sign_y > 0.0 {
                                    all_ny = false;
                                }
                                if sign_y < 0.0 {
                                    all_py = false;
                                }
                            }
                            if all_nx {
                                x0 = Some(o_to_p.x / o_to_p.z);
                            }
                            if all_px {
                                x1 = Some(o_to_p.x / o_to_p.z);
                            }
                            if all_ny {
                                y0 = Some(o_to_p.y / o_to_p.z);
                            }
                            if all_py {
                                y1 = Some(o_to_p.y / o_to_p.z);
                            }
                        }

                        let hmd_to_ccam = Matrix4::from_translation(Point3::origin() - origin);
                        let ccam_to_hmd = Matrix4::from_translation(origin - Point3::origin());

                        Things {
                            frustum: Frustum {
                                x0: x0.unwrap(),
                                x1: x1.unwrap(),
                                y0: y0.unwrap(),
                                y1: y1.unwrap(),
                                z0: z0.unwrap() - origin.z,
                                z1: z1.unwrap() - origin.z,
                            },
                            wld_to_ccam: hmd_to_ccam * parameters.wld_to_hmd,
                            ccam_to_wld: parameters.hmd_to_wld * ccam_to_hmd,
                        }
                    }
                    _ => {
                        panic!("Too many cameras for enclosed perspective clustering.");
                    }
                }
            }
        };

        self.computed = ClusterComputed {
            dimensions: dimensions.cast().unwrap(),
            frustum: things.frustum,
            wld_to_ccam: things.wld_to_ccam,
            ccam_to_wld: things.ccam_to_wld,
        };
    }

    pub fn reset(&mut self, _gl: &gl::Gl, _profiling_context: &mut ProfilingContext, parameters: ClusterParameters) {
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
    (gl: &gl::Gl, profiling_context: &mut ProfilingContext, parameters: ClusterParameters),
}
