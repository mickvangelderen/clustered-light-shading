use crate::*;

pub const MAX_LIGHTS_PER_CLUSTER: usize = 8;

pub fn compute_bounding_box<I: Iterator<Item = Matrix4<f64>>>(pos_from_clp_to_hmd_iter: I) -> frustrum::BoundingBox<f32> {
    let corners_in_clp = frustrum::Frustrum::corners_in_clp(DEPTH_RANGE);
    let mut corners_in_hmd = pos_from_clp_to_hmd_iter
        .flat_map(|pos_from_clp_to_hmd| {
            corners_in_clp
                .into_iter()
                .map(move |&p| pos_from_clp_to_hmd.transform_point(p))
        })
        .map(|p| p.cast::<f32>().unwrap());
    let first = frustrum::BoundingBox::from_point(corners_in_hmd.next().unwrap());
    corners_in_hmd.fold(first, |b, p| b.enclose(p))
}

pub fn compute_light_assignment(
    pos_from_wld_to_hmd: &Matrix4<f64>,
    cluster_bounding_box: frustrum::BoundingBox<f32>,
    point_lights: &[light::PointLight],
    cluster_side: f32,
    light_index: Option<u32>,
) -> rendering::CLSBuffer {
    let cbb_delta = cluster_bounding_box.delta();
    let cbb_dims_f32 = (cbb_delta / cluster_side).map(f32::ceil);
    // cluster side scale to world
    let cbb_side = cbb_delta.div_element_wise(cbb_dims_f32);
    // cluster side scale to cls
    let cbb_side_inv = cbb_dims_f32.div_element_wise(cbb_delta);
    let cbb_dims = cbb_dims_f32.map(|e| e as usize);
    let cbb_n = cbb_dims.x * cbb_dims.y * cbb_dims.z;

    let pos_from_hmd_to_cls = Matrix4::from_nonuniform_scale(cbb_side_inv.x, cbb_side_inv.y, cbb_side_inv.z)
        * Matrix4::from_translation(Point3::origin() - cluster_bounding_box.min);

    let pos_from_wld_to_cls: Matrix4<f64> = pos_from_hmd_to_cls.cast().unwrap() * pos_from_wld_to_hmd;
    let pos_from_cls_to_wld: Matrix4<f32> = pos_from_wld_to_cls.invert().unwrap().cast().unwrap();
    let pos_from_wld_to_cls: Matrix4<f32> = pos_from_wld_to_cls.cast().unwrap();

    let mut clustering: Vec<[u32; MAX_LIGHTS_PER_CLUSTER]> =
        (0..cbb_n).into_iter().map(|_| Default::default()).collect();

    // println!(
    //     "cluster x * y * z = {} * {} * {} = {} ({} MB)",
    //     cbb_dims.x,
    //     cbb_dims.y,
    //     cbb_dims.z,
    //     cbb_n,
    //     std::mem::size_of_val(&clustering[..]) as f32 / 1_000_000.0
    // );

    for (i, l) in point_lights.iter().enumerate() {
        if let Some(light_index) = light_index {
            if i as u32 != light_index { continue; }
        }

        let pos_in_cls = pos_from_wld_to_cls.transform_point(l.pos_in_pnt);

        let r = l.attenuation.clip_far;
        let r_sq = r * r;

        let r0 = Point3::partial_clamp_element_wise(
            (pos_in_cls - cbb_side_inv * r).map(f32::floor),
            Point3::origin(),
            Point3::from_vec(cbb_dims_f32),
        )
        .map(|e| e as usize);
        let Point3 { x: x0, y: y0, z: z0 } = r0;

        let r1 = Point3::partial_clamp_element_wise(
            (pos_in_cls).map(f32::floor),
            Point3::origin(),
            Point3::from_vec(cbb_dims_f32),
        )
        .map(|e| e as usize);
        let Point3 { x: x1, y: y1, z: z1 } = r1;

        let r2 = Point3::partial_clamp_element_wise(
            (pos_in_cls + cbb_side_inv * r).map(f32::floor),
            Point3::origin(),
            Point3::from_vec(cbb_dims_f32),
        )
        .map(|e| e as usize);
        let Point3 { x: x2, y: y2, z: z2 } = r2;

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
            let dz = closest_face_dist!(z, z1, pos_in_cls) * cbb_side.z;
            for y in y0..y2 {
                let dy = closest_face_dist!(y, y1, pos_in_cls) * cbb_side.y;
                for x in x0..x2 {
                    let dx = closest_face_dist!(x, x1, pos_in_cls) * cbb_side.x;
                    if dz * dz + dy * dy + dx * dx < r_sq {
                        // It's a hit!
                        let thing = &mut clustering[((z * cbb_dims.y) + y) * cbb_dims.x + x];

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

    let cls_buffer = rendering::CLSBuffer {
        header: rendering::CLSBufferHeader {
            dimensions: cbb_dims.map(|e: usize| e as u32).extend(MAX_LIGHTS_PER_CLUSTER as u32),
            pos_from_wld_to_cls,
            pos_from_cls_to_wld,
        },
        body: clustering,
    };

    cls_buffer
}
