use cgmath::*;

type Quad = [u32; 4];
type Tri = [u32; 3];

// Corners (8):
// 000
// 00n
// 0n0
// 0nn
// n00
// n0n
// nn0
// nnn
// Edges along X (4*n):
// 00x
// 0nx
// n0x
// nnx
// Edges along Y (4*n):
// 0y0
// 0yn
// ny0
// nyn
// Edges along Z (4*n):
// z00
// z0n
// zn0
// znn
// Faces with normal X (2*n*n):
// zy0
// zyn
// Faces with normal Y (2*n*n):
// z0x
// znx
// Faces with normal Z (2*n*n):
// 0yx
// nyx

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
enum Face {
    Negative = 0,
    Positive = 1,
}

impl Face {
    #[inline]
    fn to_f32(self) -> f32 {
        match self {
            Face::Negative => -1.0,
            Face::Positive => 1.0,
        }
    }
}

const FACES: [Face; 2] = [Face::Negative, Face::Positive];

// Corners are indexed by (face, face, face)
#[inline]
fn corner_index(z: Face, y: Face, x: Face) -> u32 {
    ((z as u32) * 2 + y as u32) * 2 + x as u32
}

// Edges are indexed by (face, face, n)
#[inline]
fn x_edge_index(n: u32, z: Face, y: Face, x: u32) -> u32 {
    8 + (2 * 2 * n) * 0 + ((z as u32) * 2 + y as u32) * n + (x - 1)
}

#[inline]
fn y_edge_index(n: u32, z: Face, y: u32, x: Face) -> u32 {
    8 + (2 * 2 * n) * 1 + ((z as u32) * 2 + x as u32) * n + (y - 1)
}

#[inline]
fn z_edge_index(n: u32, z: u32, y: Face, x: Face) -> u32 {
    8 + (2 * 2 * n) * 2 + ((y as u32) * 2 + x as u32) * n + (z - 1)
}

// Faces are indexed by (n, n, face)
#[inline]
fn x_face_index(n: u32, z: u32, y: u32, x: Face) -> u32 {
    8 + (2 * 2 * n) * 3 + (n * n * 2) * 0 + (((z - 1) * n) + (y - 1)) * 2 + x as u32
}

#[inline]
fn y_face_index(n: u32, z: u32, y: Face, x: u32) -> u32 {
    8 + (2 * 2 * n) * 3 + (n * n * 2) * 1 + (((z - 1) * n) + (x - 1)) * 2 + y as u32
}

#[inline]
fn z_face_index(n: u32, z: Face, y: u32, x: u32) -> u32 {
    8 + (2 * 2 * n) * 3 + (n * n * 2) * 2 + (((y - 1) * n) + (x - 1)) * 2 + z as u32
}

fn lerp_index_around_0(x: u32, x1: u32, y_range: f32) -> f32 {
    // (x/x1 - 0.5)*y_range
    // (x - 0.5x1)/x1*y_range
    // (2*x - x1)/(2*x1)*y_range
    ((2 * x as i32 - x1 as i32) as f32 / (2 * x1) as f32) * y_range
}

#[inline]
fn index_to_f32(n: u32, i: u32) -> f32 {
    (i as i32 * 2 - (n + 1) as i32) as f32 / (n + 1) as f32
}

#[inline]
fn to_face(n: u32, i: u32) -> Result<Face, u32> {
    if i == 0 {
        Ok(Face::Negative)
    } else if i < (n + 1) {
        Err(i)
    } else if i == (n + 1) {
        Ok(Face::Positive)
    } else {
        debug_assert!(false, "Index {} out of bounds [0, {}].", i, n + 1);
        unsafe { std::hint::unreachable_unchecked() }
    }
}

// Planes.
#[inline]
fn x_plane_index(n: u32, z: u32, y: u32, x: Face) -> u32 {
    match to_face(n, z) {
        Ok(z) => match to_face(n, y) {
            Ok(y) => corner_index(z, y, x),
            Err(y) => y_edge_index(n, z, y, x),
        },
        Err(z) => match to_face(n, y) {
            Ok(y) => z_edge_index(n, z, y, x),
            Err(y) => x_face_index(n, z, y, x),
        },
    }
}

#[inline]
fn y_plane_index(n: u32, z: u32, y: Face, x: u32) -> u32 {
    match to_face(n, z) {
        Ok(z) => match to_face(n, x) {
            Ok(x) => corner_index(z, y, x),
            Err(x) => x_edge_index(n, z, y, x),
        },
        Err(z) => match to_face(n, x) {
            Ok(x) => z_edge_index(n, z, y, x),
            Err(x) => y_face_index(n, z, y, x),
        },
    }
}

#[inline]
fn z_plane_index(n: u32, z: Face, y: u32, x: u32) -> u32 {
    match to_face(n, y) {
        Ok(y) => match to_face(n, x) {
            Ok(x) => corner_index(z, y, x),
            Err(x) => x_edge_index(n, z, y, x),
        },
        Err(y) => match to_face(n, x) {
            Ok(x) => y_edge_index(n, z, y, x),
            Err(x) => z_face_index(n, z, y, x),
        },
    }
}

pub fn generate_cubic_sphere(
    radius: f32,
    subdivisions: u32,
) -> (Vec<Vector3<f32>>, Vec<Quad>, Vec<u32>) {
    let sqrt_frac_1_3 = f32::sqrt(1.0 / 3.0);
    let acos_frac_1_3 = f32::acos(1.0 / 3.0);
    let s = sqrt_frac_1_3 * radius;

    let n = subdivisions;

    let mut positions = Vec::with_capacity((8 + 12 * n + 6 * n * n) as usize);
    // unsafe {
    //     positions.set_len(positions.capacity());
    // }

    // Corners.
    for &zf in FACES.into_iter() {
        let z = zf.to_f32() * sqrt_frac_1_3 * radius;
        for &yf in FACES.into_iter() {
            let y = yf.to_f32() * sqrt_frac_1_3 * radius;
            for &xf in FACES.into_iter() {
                let x = xf.to_f32() * sqrt_frac_1_3 * radius;
                debug_assert_eq!(positions.len(), corner_index(zf, yf, xf) as usize);
                positions.push(Vector3::new(x, y, z));
            }
        }
    }

    // Edges along x.
    for &zf in FACES.into_iter() {
        let z = zf.to_f32();
        for &yf in FACES.into_iter() {
            let y = yf.to_f32();
            for xi in 1..=n {
                let angle = index_to_f32(n, xi)*acos_frac_1_3/2.0;
                debug_assert_eq!(positions.len(), x_edge_index(n, zf, yf, xi) as usize);
                positions.push(Vector3::new(
                    radius * f32::sin(angle),
                    y * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                    z * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                ));
            }
        }
    }

    // Edges along y.
    for &zf in FACES.into_iter() {
        let z = zf.to_f32();
        for &xf in FACES.into_iter() {
            let x = xf.to_f32();
            for yi in 1..=n {
                let angle = index_to_f32(n, yi)*acos_frac_1_3/2.0;
                debug_assert_eq!(positions.len(), y_edge_index(n, zf, yi, xf) as usize);
                positions.push(Vector3::new(
                    x * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                    radius * f32::sin(angle),
                    z * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                ));
            }
        }
    }

    // Edges along z.
    for &yf in FACES.into_iter() {
        let y = yf.to_f32();
        for &xf in FACES.into_iter() {
            let x = xf.to_f32();
            for zi in 1..=n {
                let angle = index_to_f32(n, zi)*acos_frac_1_3/2.0;
                debug_assert_eq!(positions.len(), z_edge_index(n, zi, yf, xf) as usize);
                positions.push(Vector3::new(
                    x * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                    y * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                    radius * f32::sin(angle),
                ));
            }
        }
    }

    // Faces with normal x.
    for zi in 1..=n {
        let z = index_to_f32(n, zi) * s;
        for yi in 1..=n {
            let y = index_to_f32(n, yi) * s;
            for &xf in FACES.into_iter() {
                let x = xf.to_f32() * s;
                debug_assert_eq!(positions.len(), x_face_index(n, zi, yi, xf) as usize);
                positions.push(Vector3::new(x, y, z));
            }
        }
    }

    // Faces with normal y.
    for zi in 1..=n {
        let z = index_to_f32(n, zi) * s;
        for xi in 1..=n {
            let x = index_to_f32(n, xi) * s;
            for &yf in FACES.into_iter() {
                let y = yf.to_f32() * s;
                debug_assert_eq!(positions.len(), y_face_index(n, zi, yf, xi) as usize);
                positions.push(Vector3::new(x, y, z));
            }
        }
    }

    // Faces with normal z.
    for yi in 1..=n {
        let y = index_to_f32(n, yi) * s;
        for xi in 1..=n {
            let x = index_to_f32(n, xi) * s;
            for &zf in FACES.into_iter() {
                let z = zf.to_f32() * s;
                debug_assert_eq!(positions.len(), z_face_index(n, zf, yi, xi) as usize);
                positions.push(Vector3::new(x, y, z));
            }
        }
    }

    let mut faces: Vec<Quad> = Vec::with_capacity((6 * (n + 1) * (n + 1)) as usize);

    // -X
    {
        let xf = Face::Negative;
        for zi in 0..(n + 1) {
            for yi in 0..(n + 1) {
                faces.push([
                    x_plane_index(n, zi, yi, xf),
                    x_plane_index(n, zi + 1, yi, xf),
                    x_plane_index(n, zi + 1, yi + 1, xf),
                    x_plane_index(n, zi, yi + 1, xf),
                ])
            }
        }
    }

    // +X
    {
        let xf = Face::Positive;
        for zi in 0..(n + 1) {
            for yi in 0..(n + 1) {
                faces.push([
                    x_plane_index(n, zi, yi, xf),
                    x_plane_index(n, zi, yi + 1, xf),
                    x_plane_index(n, zi + 1, yi + 1, xf),
                    x_plane_index(n, zi + 1, yi, xf),
                ])
            }
        }
    }

    // -Y
    {
        let yf = Face::Negative;
        for zi in 0..(n + 1) {
            for xi in 0..(n + 1) {
                faces.push([
                    y_plane_index(n, zi, yf, xi),
                    y_plane_index(n, zi, yf, xi + 1),
                    y_plane_index(n, zi + 1, yf, xi + 1),
                    y_plane_index(n, zi + 1, yf, xi),
                ])
            }
        }
    }

    // +Y
    {
        let yf = Face::Positive;
        for zi in 0..(n + 1) {
            for xi in 0..(n + 1) {
                faces.push([
                    y_plane_index(n, zi, yf, xi),
                    y_plane_index(n, zi + 1, yf, xi),
                    y_plane_index(n, zi + 1, yf, xi + 1),
                    y_plane_index(n, zi, yf, xi + 1),
                ])
            }
        }
    }

    // -Z
    {
        let zf = Face::Negative;
        for yi in 0..(n + 1) {
            for xi in 0..(n + 1) {
                faces.push([
                    z_plane_index(n, zf, yi, xi),
                    z_plane_index(n, zf, yi + 1, xi),
                    z_plane_index(n, zf, yi + 1, xi + 1),
                    z_plane_index(n, zf, yi, xi + 1),
                ])
            }
        }
    }

    // +Z
    {
        let zf = Face::Positive;
        for yi in 0..(n + 1) {
            for xi in 0..(n + 1) {
                faces.push([
                    z_plane_index(n, zf, yi, xi),
                    z_plane_index(n, zf, yi, xi + 1),
                    z_plane_index(n, zf, yi + 1, xi + 1),
                    z_plane_index(n, zf, yi + 1, xi),
                ])
            }
        }
    }

    let objects = vec![0, faces.len() as u32];

    (positions, faces, objects)
}

#[derive(Debug)]
enum Poly {
    None,
    Quad(Quad),
    Tri(Tri),
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
