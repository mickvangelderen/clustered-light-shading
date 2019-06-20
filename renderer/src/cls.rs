use crate::*;

pub const MAX_LIGHTS_PER_CLUSTER: usize = 8;

fn compute_bounding_box(pos_from_clp_to_hmd: &[Matrix4<f64>]) -> BoundingBox<f32> {
    let corners_in_clp = Frustrum::corners_in_clp(DEPTH_RANGE);
    let mut corners_in_hmd = pos_from_clp_to_hmd
        .iter()
        .flat_map(|pos_from_clp_to_hmd| {
            corners_in_clp
                .into_iter()
                .map(move |&p| pos_from_clp_to_hmd.transform_point(p))
        })
        .map(|p| p.cast::<f32>().unwrap());
    let first = BoundingBox::from_point(corners_in_hmd.next().unwrap());
    corners_in_hmd.fold(first, |b, p| b.enclose(p))
}

pub fn compute_light_assignment(
    pos_from_clp_to_hmd: &[Matrix4<f64>],
    pos_from_wld_to_hmd: Matrix4<f64>,
    pos_from_hmd_to_wld: Matrix4<f64>,
    point_lights: &[light::PointLight],
    configuration: &configuration::ClusteredLightShading,
) -> rendering::ClusterData {
    // Get configuration.
    let cluster_side_max = configuration.cluster_side;
    let light_index = configuration.light_index;

    let cluster_bounding_box = compute_bounding_box(pos_from_clp_to_hmd);

    let cbb_delta = cluster_bounding_box.delta();
    let dimensions_f32 = (cbb_delta / cluster_side_max).map(f32::ceil);
    let scale_from_cls_to_hmd = cbb_delta.div_element_wise(dimensions_f32);
    let scale_from_hmd_to_cls = dimensions_f32.div_element_wise(cbb_delta);
    let dimensions_u32 = dimensions_f32.map(|e| e as u32);
    let cluster_count = dimensions_u32.product();

    let pos_from_hmd_to_cls: Matrix4<f64> =
        Matrix4::from_scale_vector(scale_from_hmd_to_cls.cast().unwrap())
         * Matrix4::from_translation(-cluster_bounding_box.min.cast().unwrap().to_vec());
    let pos_from_cls_to_hmd: Matrix4<f64> =
        Matrix4::from_translation(cluster_bounding_box.min.cast().unwrap().to_vec())*
        Matrix4::from_scale_vector(scale_from_cls_to_hmd.cast().unwrap());

    let pos_from_wld_to_cls: Matrix4<f32> = (pos_from_hmd_to_cls * pos_from_wld_to_hmd).cast().unwrap();
    let pos_from_cls_to_wld: Matrix4<f32> = (pos_from_hmd_to_wld * pos_from_cls_to_hmd).cast().unwrap();

    let mut clustering: Vec<[u32; MAX_LIGHTS_PER_CLUSTER]> =
        (0..cluster_count).into_iter().map(|_| Default::default()).collect();

    // println!(
    //     "cluster x * y * z = {} * {} * {} = {} ({} MB)",
    //     bounding_box_scale_in_cls_usize.x,
    //     bounding_box_scale_in_cls_usize.y,
    //     bounding_box_scale_in_cls_usize.z,
    //     cbb_n,
    //     std::mem::size_of_val(&clustering[..]) as f32 / 1_000_000.0
    // );

    for (i, l) in point_lights.iter().enumerate() {
        if let Some(light_index) = light_index {
            if i as u32 != light_index {
                continue;
            }
        }

        let pos_in_cls = pos_from_wld_to_cls.transform_point(l.pos_in_wld);

        let r = l.attenuation.clip_far;
        let r_sq = r * r;

        let minima = Point3::partial_clamp_element_wise(
            (pos_in_cls - scale_from_hmd_to_cls * r).map(f32::floor),
            Point3::origin(),
            Point3::from_vec(dimensions_f32),
        )
        .map(|e| e as u32);

        let centers = Point3::partial_clamp_element_wise(
            (pos_in_cls).map(f32::floor),
            Point3::origin(),
            Point3::from_vec(dimensions_f32),
        )
        .map(|e| e as u32);

        let maxima = Point3::partial_clamp_element_wise(
            (pos_in_cls + scale_from_hmd_to_cls * r).map(f32::ceil),
            Point3::origin(),
            Point3::from_vec(dimensions_f32),
        )
        .map(|e| e as u32);

        let Point3 { x: x0, y: y0, z: z0 } = minima;
        let Point3 { x: x1, y: y1, z: z1 } = centers;
        let Point3 { x: x2, y: y2, z: z2 } = maxima;

        // NOTE: We must clamp as f32 because the value might actually overflow.

        macro_rules! closest_face_dist {
            ($x: ident, $x1: ident, $pos: ident) => {
                if $x < $x1 {
                    ($x + 1) as f32 - $pos.$x
                } else if $x > $x1 {
                    $x as f32 - $pos.$x
                } else {
                    0.0
                }
            };
        }

        for z in z0..z2 {
            let dz = closest_face_dist!(z, z1, pos_in_cls) * scale_from_cls_to_hmd.z;
            for y in y0..y2 {
                let dy = closest_face_dist!(y, y1, pos_in_cls) * scale_from_cls_to_hmd.y;
                for x in x0..x2 {
                    let dx = closest_face_dist!(x, x1, pos_in_cls) * scale_from_cls_to_hmd.x;
                    if dz * dz + dy * dy + dx * dx < r_sq {
                        // It's a hit!
                        let index = ((z * dimensions_u32.y) + y) * dimensions_u32.x + x;
                        let thing = &mut clustering[index as usize];

                        thing[0] += 1;
                        let offset = thing[0] as usize;
                        if offset < thing.len() {
                            thing[offset] = i as u32;
                        } else {
                            eprintln!("Overflowing clustered light assignment!");
                        }
                    }
                }
            }
        }
    }

    rendering::ClusterData {
        header: rendering::ClusterHeader {
            dimensions: dimensions_u32.extend(MAX_LIGHTS_PER_CLUSTER as u32),
            pos_from_wld_to_cls,
            pos_from_cls_to_wld,
        },
        body: clustering,
    }
}
