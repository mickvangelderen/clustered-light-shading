use crate::*;
use cluster_space_buffer::ClusterSpaceCoefficients;
use renderer::configuration::ClusteringProjection;
use renderer::*;

impl_enum_and_enum_map! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
    enum ClusterStage => struct ClusterStages {
        Cluster => cluster,
        CompactClusters => compact_clusters,
        TransformLights => transform_lights,
        CountLights => count_lights,
        LightOffsets => light_offsets,
        AssignLights => assign_lights,
    }
}

impl ClusterStage {
    pub const VALUES: [ClusterStage; 6] = [
        ClusterStage::Cluster,
        ClusterStage::CompactClusters,
        ClusterStage::TransformLights,
        ClusterStage::CountLights,
        ClusterStage::LightOffsets,
        ClusterStage::AssignLights,
    ];

    pub fn title(self) -> &'static str {
        match self {
            ClusterStage::Cluster => "cluster",
            ClusterStage::CompactClusters => "compact_clusters",
            ClusterStage::TransformLights => "transform_lights",
            ClusterStage::CountLights => "count_lights",
            ClusterStage::LightOffsets => "compact_lights",
            ClusterStage::AssignLights => "assign_lights",
        }
    }
}

#[derive(Debug)]
pub struct ClusterParameters {
    pub configuration: configuration::ClusteredLightShadingConfiguration,
    pub clu_ori_to_wld: Matrix4<f64>,
}

#[derive(Debug)]
pub struct ClusterComputed {
    pub dimensions: Vector3<u32>,
    pub frustum: Frustum<f64>, // useful for finding perspective transform frustum planes for intersection tests in shaders.

    pub wld_to_clu_cam: Matrix4<f64>,
    pub clu_cam_to_wld: Matrix4<f64>,

    pub cam_to_clp: ClusterSpaceCoefficients,
    pub clp_to_cam: ClusterSpaceCoefficients,
}

impl std::default::Default for ClusterComputed {
    fn default() -> Self {
        Self {
            dimensions: Vector3::zero(),
            frustum: Frustum::<f64>::zero(),
            wld_to_clu_cam: Matrix4::identity(),
            clu_cam_to_wld: Matrix4::identity(),
            cam_to_clp: ClusterSpaceCoefficients::default(),
            clp_to_cam: ClusterSpaceCoefficients::default(),
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
    pub cluster_maybe_active_cluster_indices_buffer: DynamicBuffer,

    pub active_cluster_cluster_indices_buffer: DynamicBuffer,
    pub active_cluster_light_counts_buffer: DynamicBuffer,
    pub active_cluster_light_offsets_buffer: DynamicBuffer,

    pub light_xyzr_buffer_ring: Ring3<StorageBuffer<StorageBufferKindWO>>,
    pub light_indices_buffer: DynamicBuffer,

    pub offset_buffer: DynamicBuffer,
    pub draw_commands_buffer: DynamicBuffer,
    pub compute_commands_buffer: DynamicBuffer,

    pub profiling_cluster_buffer: DynamicBuffer,
    pub profilers: ClusterStages<SampleIndex>,

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
            cluster_maybe_active_cluster_indices_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "cluster_maybe_active_cluster_indices");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_clusters as usize);
                buffer
            },
            active_cluster_cluster_indices_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "active_cluster_cluster_indices");
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
            light_xyzr_buffer_ring: Ring3::new(|| unsafe { StorageBuffer::new(gl) }),
            light_indices_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "light_indices");
                buffer.ensure_capacity(gl, std::mem::size_of::<u32>() * cfg.max_light_indices as usize);
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
            profiling_cluster_buffer: unsafe {
                let mut buffer = Buffer::new(gl);
                gl.buffer_label(&buffer, "profiling_cluster_buffer");
                buffer.ensure_capacity(gl, std::mem::size_of::<profiling::ClusterBuffer>());
                buffer
            },
            profilers: ClusterStages::new(|stage| profiling_context.add_sample(stage.title())),

            camera_resources_pool: ClusterCameraResourcesPool::new(),
            parameters,
            computed: Default::default(),
        }
    }

    pub fn recompute(&mut self) {
        let parameters = &self.parameters;
        let cfg = &parameters.configuration;

        let mut computed = match cfg.projection {
            ClusteringProjection::Orthographic => {
                // Compute bounding box of all camera frustum corners.
                let corners_in_clp = RENDER_RANGE.vertices();
                let range = Range3::<f64>::from_points({
                    self.camera_resources_pool.used_slice().iter().flat_map(|camera| {
                        let ren_clp_to_clu_ori = (camera.parameters.camera.wld_to_clp * parameters.clu_ori_to_wld)
                            .invert()
                            .unwrap();
                        corners_in_clp
                            .iter()
                            .map(move |&p| ren_clp_to_clu_ori.transform_point(p))
                    })
                })
                .unwrap();

                let dimensions = range
                    .delta()
                    .div_element_wise(Vector3::from(cfg.orthographic_sides.to_array()))
                    .map(f64::ceil);

                let frustum = Frustum::from_range(&range);

                ClusterComputed {
                    dimensions: dimensions.cast::<u32>().unwrap(),
                    frustum,
                    wld_to_clu_cam: parameters.clu_ori_to_wld.invert().unwrap(),
                    clu_cam_to_wld: parameters.clu_ori_to_wld,
                    cam_to_clp: ClusterSpaceCoefficients::orthographic(&frustum, dimensions),
                    clp_to_cam: ClusterSpaceCoefficients::inverse_orthographic(&frustum, dimensions),
                }
            }
            ClusteringProjection::Perspective => {
                let cameras = self.camera_resources_pool.used_slice();

                match cameras.len() {
                    1 => {
                        let camera = &cameras[0];

                        let dims = camera.parameters.frame_dims;
                        let f = camera.parameters.camera.frustum;

                        let px = cfg.perspective_pixels.x;
                        let py = cfg.perspective_pixels.y;

                        let x_per_c = f.dx() * px as f64 / dims.x as f64;
                        let y_per_c = f.dy() * py as f64 / dims.y as f64;
                        let d_per_c = (x_per_c + y_per_c) * 0.5;
                        let dimensions = Vector3::new(
                            dims.x.ceiled_div(px as i32) as f64,
                            dims.y.ceiled_div(py as i32) as f64,
                            (f.z0 / f.z1).log(1.0 + d_per_c).ceil(),
                        );

                        // We adjust the frustum to make clusters line up nicely
                        // with pixels in the framebuffer..
                        let frustum = Frustum {
                            x0: f.x0,
                            x1: f.x0 + dimensions.x * x_per_c,
                            y0: f.y0,
                            y1: f.y0 + dimensions.y * y_per_c,
                            z0: f.z1 * (1.0 + d_per_c).powi(dimensions.z as i32),
                            z1: f.z1,
                        };

                        cgmath::assert_relative_eq!(camera.parameters.camera.cam_to_wld, parameters.clu_ori_to_wld);

                        ClusterComputed {
                            dimensions: dimensions.cast::<u32>().unwrap(),
                            frustum,
                            wld_to_clu_cam: parameters.clu_ori_to_wld.invert().unwrap(),
                            clu_cam_to_wld: parameters.clu_ori_to_wld,
                            cam_to_clp: ClusterSpaceCoefficients::perspective(&frustum, dimensions, d_per_c),
                            clp_to_cam: ClusterSpaceCoefficients::inverse_perspective(&frustum, dimensions, d_per_c),
                        }
                    }
                    2 => {
                        let far_pos_in_clp = RENDER_RANGE.far_vertices();
                        let near_pos_in_clp = RENDER_RANGE.near_vertices();

                        let far_pos_in_clu_ori: Vec<Point3<f64>> = self
                            .camera_resources_pool
                            .used_slice()
                            .iter()
                            .flat_map(|camera| {
                                let ren_clp_to_clu_ori = (camera.parameters.camera.wld_to_clp
                                    * parameters.clu_ori_to_wld)
                                    .invert()
                                    .unwrap();
                                far_pos_in_clp
                                    .iter()
                                    .map(move |&pos_in_clp| ren_clp_to_clu_ori.transform_point(pos_in_clp))
                            })
                            .collect();

                        let near_pos_in_clu_ori: Vec<Point3<f64>> = self
                            .camera_resources_pool
                            .used_slice()
                            .iter()
                            .flat_map(|camera| {
                                let ren_clp_to_clu_ori = (camera.parameters.camera.wld_to_clp
                                    * parameters.clu_ori_to_wld)
                                    .invert()
                                    .unwrap();
                                near_pos_in_clp
                                    .iter()
                                    .map(move |&pos_in_clp| ren_clp_to_clu_ori.transform_point(pos_in_clp))
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
                        for (fi, &f) in far_pos_in_clu_ori.iter().enumerate() {
                            for (ni, &n) in near_pos_in_clu_ori.iter().enumerate() {
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
                                    for &p in near_pos_in_clu_ori.iter().chain(far_pos_in_clu_ori.iter()) {
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
                        for (fi, &f) in far_pos_in_clu_ori.iter().enumerate() {
                            for (ni, &n) in near_pos_in_clu_ori.iter().enumerate() {
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
                                for &p in far_pos_in_clu_ori.iter().chain(near_pos_in_clu_ori.iter()) {
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
                        for &p in far_pos_in_clu_ori.iter().chain(near_pos_in_clu_ori.iter()) {
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
                            for &q in far_pos_in_clu_ori.iter().chain(near_pos_in_clu_ori.iter()) {
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

                        let clu_cam_to_clu_ori = Matrix4::from_translation(origin - Point3::origin());

                        let f = Frustum {
                            x0: x0.unwrap(),
                            x1: x1.unwrap(),
                            y0: y0.unwrap(),
                            y1: y1.unwrap(),
                            z0: z0.unwrap() - origin.z,
                            z1: z1.unwrap() - origin.z,
                        };

                        let avg_x_per_c = {
                            let sum: f64 = self
                                .camera_resources_pool
                                .used_slice()
                                .iter()
                                .map(|camera| {
                                    let d = &camera.parameters.frame_dims;
                                    let f = &camera.parameters.camera.frustum;
                                    f.dx() / d.x as f64
                                })
                                .sum();
                            sum / self.camera_resources_pool.used_slice().len() as f64
                        } * cfg.perspective_pixels.x as f64;

                        let avg_y_per_c = {
                            let sum: f64 = self
                                .camera_resources_pool
                                .used_slice()
                                .iter()
                                .map(|camera| {
                                    let d = &camera.parameters.frame_dims;
                                    let f = &camera.parameters.camera.frustum;
                                    f.dy() / d.y as f64
                                })
                                .sum();
                            sum / self.camera_resources_pool.used_slice().len() as f64
                        } * cfg.perspective_pixels.y as f64;

                        let cls_x = (f.dx() / avg_x_per_c).ceil();
                        let cls_y = (f.dy() / avg_y_per_c).ceil();

                        let x_per_c = f.dx() / cls_x;
                        let y_per_c = f.dy() / cls_y;
                        let d_per_c = (x_per_c + y_per_c) * 0.5;

                        let cls_z = (f.z0 / f.z1).log(1.0 + d_per_c).ceil();

                        let dimensions = Vector3::new(cls_x, cls_y, cls_z);

                        let frustum = Frustum {
                            x0: f.x0,
                            x1: f.x0 + cls_x * x_per_c,
                            y0: f.y0,
                            y1: f.y0 + cls_y * y_per_c,
                            z0: f.z1 * (1.0 + d_per_c).powi(cls_z as i32),
                            z1: f.z1,
                        };

                        let clu_cam_to_wld = parameters.clu_ori_to_wld * clu_cam_to_clu_ori;

                        ClusterComputed {
                            dimensions: dimensions.cast::<u32>().unwrap(),
                            frustum,
                            wld_to_clu_cam: clu_cam_to_wld.invert().unwrap(),
                            clu_cam_to_wld,
                            cam_to_clp: ClusterSpaceCoefficients::perspective(&frustum, dimensions, d_per_c),
                            clp_to_cam: ClusterSpaceCoefficients::inverse_perspective(&frustum, dimensions, d_per_c),
                        }
                    }
                    _ => {
                        panic!("Too many cameras for enclosed perspective clustering.");
                    }
                }
            }
        };

        for i in 0..3 {
            if computed.dimensions[i] < 1 {
                computed.dimensions[i] = 1;
            }
            assert!(computed.dimensions[i] <= 1024);
        }

        self.computed = computed;
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
