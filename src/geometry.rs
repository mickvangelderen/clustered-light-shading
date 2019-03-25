use cgmath::*;

type Quad = [u32; 4];
type Tri = [u32; 3];

#[derive(Debug)]
enum Poly {
    None,
    Quad(Quad),
    Tri(Tri),
}

impl Poly {
    pub fn is_tri(&self) -> bool {
        match self {
            Poly::Tri(_) => true,
            _ => false,
        }
    }
}

pub fn generate_iso_sphere(
    scale: f32,
    subdivisions: u32,
) -> (Vec<Vector3<f32>>, Vec<Tri>, Vec<u32>) {
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

    // Upper bound.
    let mut triangles: Vec<Tri> = Vec::with_capacity(4 * 3usize.pow(subdivisions + 1));
    triangles.extend([[0, 1, 2], [2, 3, 0], [0, 3, 1], [1, 3, 2]].into_iter());

    let mut objects: Vec<u32> = Vec::with_capacity(subdivisions as usize + 2);
    objects.extend([0, triangles.len() as u32].into_iter());

    for subdivision in 0..subdivisions {
        let triangle_start = objects[subdivision as usize] as usize;
        let triangle_end = objects[subdivision as usize + 1] as usize;

        // println!("triangles {}..{}", triangle_start, triangle_end);
        // for i in triangle_start..triangle_end {
        //     println!("{:02}: {:?}", i, triangles[i]);
        // }

        for t1_idx in triangle_start..triangle_end {
            let t1 = triangles[t1_idx];
            let t1_center =
                (positions[t1[0] as usize] + positions[t1[1] as usize] + positions[t1[2] as usize])
                    .normalize_to(scale);

            let mut poly = Poly::Tri(t1);

            let mut largest_diff = 0.0;

            for t2_idx in triangle_start..triangle_end {
                if t1_idx == t2_idx {
                    continue;
                }
                let t2 = triangles[t2_idx];
                let t2_center = (positions[t2[0] as usize]
                    + positions[t2[1] as usize]
                    + positions[t2[2] as usize])
                    .normalize_to(scale);

                for i in 0..3 {
                    let next_i = (i + 1) % 3;
                    for j in 0..3 {
                        let next_j = (j + 1) % 3;
                        if t1[i] == t2[next_j] && t1[next_i] == t2[j] {
                            let shared_center =
                                positions[t1[i] as usize] + positions[t1[next_i] as usize];
                            let cross_center = t1_center + t2_center;
                            let diff = cross_center.magnitude2() - shared_center.magnitude2();
                            if diff > largest_diff {
                                largest_diff = diff;
                                let prev_i = (i + 2) % 3;
                                let prev_j = (j + 2) % 3;
                                if t1_idx < t2_idx {
                                    poly = Poly::Quad([t1[prev_i], t1[i], t2[prev_j], t2[j]]);
                                } else {
                                    poly = Poly::None;
                                }
                            }
                        }
                    }
                }
            }

            match poly {
                Poly::None => {}
                Poly::Tri([a, b, c]) => {
                    let d = positions.len() as u32;
                    positions.push(
                        (positions[a as usize] + positions[b as usize] + positions[c as usize])
                            .normalize_to(scale),
                    );
                    triangles.push([a, b, d]);
                    triangles.push([b, c, d]);
                    triangles.push([c, a, d]);
                }
                Poly::Quad([a, b, c, d]) => {
                    let e = positions.len() as u32;
                    positions.push(
                        (positions[a as usize]
                            + positions[b as usize]
                            + positions[c as usize]
                            + positions[d as usize])
                            .normalize_to(scale),
                    );
                    triangles.push([a, b, e]);
                    triangles.push([b, c, e]);
                    triangles.push([c, d, e]);
                    triangles.push([d, a, e]);
                }
            }
        }

        objects.push(triangles.len() as u32);
    }

    (positions, triangles, objects)
}

trait FindOrPushGetIndex<T> {
    fn find_or_push_get_index(&mut self, value: T) -> usize;
}

impl<T: Copy + PartialEq> FindOrPushGetIndex<T> for Vec<T> {
    fn find_or_push_get_index(&mut self, value: T) -> usize {
        self.iter()
            .position(|&item| item == value)
            .unwrap_or_else(|| {
                let position = self.len();
                self.push(value);
                position
            })
    }
}

trait PushGetIndex<T> {
    fn push_get_index(&mut self, value: T) -> usize;
}

impl<T> PushGetIndex<T> for Vec<T> {
    fn push_get_index(&mut self, value: T) -> usize {
        let index = self.len();
        self.push(value);
        index
    }
}

pub fn generate_cubic_sphere(
    radius: f32,
    subdivisions: u32,
) -> (Vec<Vector3<f32>>, Vec<Quad>, Vec<u32>) {
    let s = (radius as f64 * f64::sqrt(1.0/3.0)) as f32;

    let mut positions = vec![
        Vector3::new(-s, -s, -s),
        Vector3::new(s, -s, -s),
        Vector3::new(-s, s, -s),
        Vector3::new(s, s, -s),
        Vector3::new(-s, -s, s),
        Vector3::new(s, -s, s),
        Vector3::new(-s, s, s),
        Vector3::new(s, s, s),
    ];

    let mut quads = vec![
        [5, 7, 6, 4],
        [5, 1, 3, 7],
        [5, 4, 0, 1],
        [2, 3, 1, 0],
        [2, 6, 7, 3],
        [2, 0, 4, 6],
    ];

    let mut objects = vec![0, quads.len() as u32];

    for subdivision in 0..subdivisions {
        let start = objects[subdivision as usize];
        let end = objects[(subdivision + 1) as usize];
        for qi in start..end {
            let q = quads[qi as usize];

            let i_a = q[0];
            let i_b = q[1];
            let i_c = q[2];
            let i_d = q[3];

            let v_a = positions[i_a as usize];
            let v_b = positions[i_b as usize];
            let v_c = positions[i_c as usize];
            let v_d = positions[i_d as usize];

            // Vertices on the side might have already been added.
            let i_ab = positions.find_or_push_get_index((v_a + v_b).normalize_to(radius)) as u32;
            let i_bc = positions.find_or_push_get_index((v_b + v_c).normalize_to(radius)) as u32;
            let i_cd = positions.find_or_push_get_index((v_c + v_d).normalize_to(radius)) as u32;
            let i_da = positions.find_or_push_get_index((v_d + v_a).normalize_to(radius)) as u32;

            // Vertex in the center only connect to the new quads.
            let i_abcd = positions.push_get_index((v_a + v_b + v_c + v_d).normalize_to(radius)) as u32;

            quads.extend([
                [i_abcd, i_da, i_a, i_ab],
                [i_abcd, i_ab, i_b, i_bc],
                [i_abcd, i_bc, i_c, i_cd],
                [i_abcd, i_cd, i_d, i_da],
            ].into_iter());
        }

        objects.push(quads.len() as u32);
    }

    (positions, quads, objects)
}

