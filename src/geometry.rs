use cgmath::*;

pub fn generate_iso_sphere(
    scale: f32,
    subdivisions: u32,
) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>, Vec<u32>) {
    let mut positions = vec![
        Vector3::new(0.0, scale, 0.0),
        Vector3::new(0.0, -(1.0 / 3.0) * scale, f32::sqrt(8.0 / 9.0) * scale),
        Vector3::new(
            -f32::sqrt(2.0 / 3.0) * scale,
            -(1.0 / 3.0) * scale,
            -f32::sqrt(2.0 / 9.0) * scale,
        ),
        Vector3::new(
            f32::sqrt(2.0 / 3.0) * scale,
            -(1.0 / 3.0) * scale,
            -f32::sqrt(2.0 / 9.0) * scale,
        ),
    ];

    let mut triangles: Vec<[u32; 3]> = Vec::with_capacity(4 * 3usize.pow(subdivisions + 1));
    triangles.extend([[0, 1, 2], [2, 3, 0], [0, 3, 1], [1, 3, 2]].into_iter());

    let mut objects: Vec<u32> = Vec::with_capacity(subdivisions as usize + 2);
    objects.extend([0, triangles.len() as u32].into_iter());

    for subdivision in 0..subdivisions {
        let triangle_start = objects[subdivision as usize] as usize;
        let triangle_end = objects[subdivision as usize + 1] as usize;

        let mut shared_normal = false;

        for t1_idx in triangle_start..triangle_end {
            let t1 = triangles[t1_idx];
            let t1_p0 = positions[t1[0] as usize];
            let t1_p1 = positions[t1[1] as usize];
            let t1_p2 = positions[t1[2] as usize];
            let t1_n = (t1_p1 - t1_p0).cross(t1_p2 - t1_p1);

            for t2_idx in triangle_start..triangle_end {
                if t1_idx == t2_idx { continue; }
                let t2 = triangles[t2_idx];
                let t2_p0 = positions[t2[0] as usize];
                let t2_p1 = positions[t2[1] as usize];
                let t2_p2 = positions[t2[2] as usize];
                let t2_n = (t2_p1 - t2_p0).cross(t2_p2 - t2_p1);


                if cgmath::ulps_eq!(t1_n, t2_n, max_ulps = 14) {
                    println!("{:?}", t1_n - t2_n);
                    println!("shared normal {}, {}", t1_idx, t2_idx);
                    shared_normal = true;
                    // Faces share the same normal.
                    let v_center = positions.len() as u32;
                    positions.push((t1_p0 + t1_p1 + t1_p2 + t2_p0 + t2_p1 + t2_p2).normalize_to(scale));

                    for i in 0..3 {
                        let next_i = (i + 1) % 3;
                        // Figure out if (i, next_i) is shared.
                        let mut shared_edge = false;
                        for j in 0..3 {
                            let next_j = (j + 2) % 3; // (j - 1) % 3 -> (j - 1 + 3) % 3 -> (j + 2) % 3.
                            if t1[i] == t2[j] && t1[next_i] == t2[next_j] {
                                println!("shared edge {:?}, {:?}", (t1[i], t1[next_i]), (t2[j], t2[next_j]));
                                shared_edge = true;
                                break;
                            }
                        }

                        // Only emit triangles for edges that are not shared.
                        if !shared_edge {
                            triangles.push([t1[i], t1[next_i], v_center]);
                        }
                    }
                }
            }

            if !shared_normal {
                // Get the new position index.
                let v_center = positions.len() as u32;
                // Add the new position at the averaged position with a magnitude of
                // scale.
                positions.push((t1_p0 + t1_p1 + t1_p2).normalize_to(scale));
                // Add two more.
                triangles.push([t1[0], t1[1], v_center]);
                triangles.push([t1[1], t1[2], v_center]);
                triangles.push([t1[2], t1[0], v_center]);
            }
        }

        objects.push(triangles.len() as u32);
    }

    (positions, triangles, objects)
}
