use crate::*;

pub const CLUSTER_CAPACITY: usize = 8;

#[derive(Debug)]
#[repr(C)]
pub struct ClusterHeader {
    pub dimensions: Vector4<u32>,
    pub wld_to_cls: Matrix4<f32>,
    pub cls_to_wld: Matrix4<f32>,
}

#[derive(Debug)]
pub struct ClusterBuffer {
    pub header: ClusterHeader,
    pub body: Vec<[u32; CLUSTER_CAPACITY]>,
}

pub const CLUSTER_BUFFER_DECLARATION: &'static str = r"
layout(std430, binding = CLUSTER_BUFFER_BINDING) buffer ClusterBuffer {
    uvec4 cluster_dims;
    mat4 wld_to_cls;
    mat4 cls_to_wld;
    uint clusters[];
};
";

fn compute_bounding_box<I>(clp_to_hmd: I) -> BoundingBox<f64>
where
    I: IntoIterator<Item = Matrix4<f64>>,
{
    let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
    let mut corners_in_hmd = clp_to_hmd
        .into_iter()
        .flat_map(|clp_to_hmd| corners_in_clp.into_iter().map(move |&p| clp_to_hmd.transform_point(p)))
        .map(|p| p.cast::<f64>().unwrap());
    let first = BoundingBox::from_point(corners_in_hmd.next().unwrap());
    corners_in_hmd.fold(first, |b, p| b.enclose(p))
}

#[derive(Debug)]
pub struct ClusterData {
    pub dimensions: Vector3<f64>,
    pub cls_origin: Point3<f64>,
    pub scale_from_cls_to_hmd: Vector3<f64>,
    pub scale_from_hmd_to_cls: Vector3<f64>,

    pub cls_to_wld: Matrix4<f64>,
    pub wld_to_cls: Matrix4<f64>,
}

impl ClusterData {
    pub fn new<I>(
        cfg: &configuration::ClusteredLightShading,
        clp_to_hmd: I,
        wld_to_hmd: Matrix4<f64>,
        hmd_to_wld: Matrix4<f64>,
    ) -> Self
    where
        I: IntoIterator<Item = Matrix4<f64>>,
    {
        let bb = compute_bounding_box(clp_to_hmd);

        let bb_delta = bb.delta();
        let dimensions = (bb_delta / cfg.cluster_side as f64).map(f64::ceil);

        let cls_origin = bb.min;
        let scale_from_cls_to_hmd = bb_delta.div_element_wise(dimensions);
        let scale_from_hmd_to_cls = dimensions.div_element_wise(bb_delta);

        let cls_to_hmd: Matrix4<f64> =
            Matrix4::from_translation(cls_origin.to_vec()) * Matrix4::from_scale_vector(scale_from_cls_to_hmd);
        let hmd_to_cls: Matrix4<f64> =
            Matrix4::from_scale_vector(scale_from_hmd_to_cls) * Matrix4::from_translation(-cls_origin.to_vec());

        Self {
            dimensions,
            cls_origin,
            scale_from_cls_to_hmd,
            scale_from_hmd_to_cls,

            cls_to_wld: hmd_to_wld * cls_to_hmd,
            wld_to_cls: hmd_to_cls * wld_to_hmd,
        }
    }
}

pub struct ClusterResources {
    pub buffer_name: gl::BufferName,
    pub clusters: Vec<[u32; CLUSTER_CAPACITY]>,
    // pub cluster_lengths: Vec<u32>,
    // pub cluster_offsets: Vec<u32>,
    // pub light_indices: Vec<u32>,
}

impl ClusterResources {
    pub fn new(gl: &gl::Gl) -> Self {
        Self {
            buffer_name: unsafe { gl.create_buffer() },
            clusters: Vec::new(),
        }
    }
}

impl ClusterResources {
    pub fn compute_and_upload(
        &mut self,
        gl: &gl::Gl,
        cfg: &configuration::ClusteredLightShading,
        space: &ClusterData,
        point_lights: &[light::PointLight],
    ) {
        let ClusterData {
            dimensions,
            scale_from_cls_to_hmd,
            scale_from_hmd_to_cls,
            wld_to_cls,
            cls_to_wld,
            ..
        } = *space;

        let dimensions_u32 = dimensions.cast::<u32>().unwrap();
        let cluster_count = dimensions_u32.product();

        self.clusters.clear();
        self.clusters.resize_with(cluster_count as usize, Default::default);

        for (i, l) in point_lights.iter().enumerate() {
            if let Some(light_index) = cfg.light_index {
                if i as u32 != light_index {
                    continue;
                }
            }

            let pos_in_cls = wld_to_cls.transform_point(l.pos_in_wld.cast::<f64>().unwrap());

            let r = l.attenuation.clip_far as f64;
            let r_sq = r * r;

            let minima = Point3::partial_clamp_element_wise(
                (pos_in_cls - scale_from_hmd_to_cls * r).map(f64::floor),
                Point3::origin(),
                Point3::from_vec(dimensions),
            )
            .map(|e| e as u32);

            let centers = Point3::partial_clamp_element_wise(
                (pos_in_cls).map(f64::floor),
                Point3::origin(),
                Point3::from_vec(dimensions),
            )
            .map(|e| e as u32);

            let maxima = Point3::partial_clamp_element_wise(
                (pos_in_cls + scale_from_hmd_to_cls * r).map(f64::ceil),
                Point3::origin(),
                Point3::from_vec(dimensions),
            )
            .map(|e| e as u32);

            let Point3 { x: x0, y: y0, z: z0 } = minima;
            let Point3 { x: x1, y: y1, z: z1 } = centers;
            let Point3 { x: x2, y: y2, z: z2 } = maxima;

            // NOTE: We must clamp as f64 because the value might actually overflow.

            macro_rules! closest_face_dist {
                ($x: ident, $x1: ident, $pos: ident) => {
                    if $x < $x1 {
                        ($x + 1) as f64 - $pos.$x
                    } else if $x > $x1 {
                        $x as f64 - $pos.$x
                    } else {
                        0.0
                    }
                };
            }

            let mut overflow_count = 0usize;

            for z in z0..z2 {
                let dz = closest_face_dist!(z, z1, pos_in_cls) * scale_from_cls_to_hmd.z;
                for y in y0..y2 {
                    let dy = closest_face_dist!(y, y1, pos_in_cls) * scale_from_cls_to_hmd.y;
                    for x in x0..x2 {
                        let dx = closest_face_dist!(x, x1, pos_in_cls) * scale_from_cls_to_hmd.x;
                        if dz * dz + dy * dy + dx * dx < r_sq {
                            // It's a hit!
                            let index = ((z * dimensions_u32.y) + y) * dimensions_u32.x + x;
                            let thing = &mut self.clusters[index as usize];

                            thing[0] += 1;
                            let offset = thing[0] as usize;
                            if offset < thing.len() {
                                thing[offset] = i as u32;
                            } else {
                                overflow_count += 1;
                            }
                        }
                    }
                }
            }

            if overflow_count > 0 {
                warn!("Overflowing light assignment: {}", overflow_count);
            }
        }

        unsafe {
            let header = ClusterHeader {
                dimensions: dimensions_u32.extend(CLUSTER_CAPACITY as u32),
                wld_to_cls: wld_to_cls.cast().unwrap(),
                cls_to_wld: cls_to_wld.cast().unwrap(),
            };

            let header_bytes = header.value_as_bytes();
            let body_bytes = self.clusters.vec_as_bytes();

            gl.named_buffer_reserve(self.buffer_name, header_bytes.len() + body_bytes.len(), gl::STREAM_DRAW);
            gl.named_buffer_sub_data(self.buffer_name, 0, header_bytes);
            gl.named_buffer_sub_data(self.buffer_name, header_bytes.len(), body_bytes);
            gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, CLUSTER_BUFFER_BINDING, self.buffer_name);
        }
    }
}
