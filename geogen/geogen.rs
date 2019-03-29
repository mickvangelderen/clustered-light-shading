use std::f32::consts::*;

type Quad = [u32; 4];
type Tri = [u32; 3];

#[derive(Debug, Clone, Copy)]
enum Basis {
    XYZ,
    YZX,
    ZXY,
}

impl Basis {
    #[inline]
    fn rotate_cw(self) -> Self {
        match self {
            Basis::XYZ => Basis::YZX,
            Basis::YZX => Basis::ZXY,
            Basis::ZXY => Basis::XYZ,
        }
    }

    #[inline]
    fn rotate_ccw(self) -> Self {
        match self {
            Basis::XYZ => Basis::ZXY,
            Basis::YZX => Basis::XYZ,
            Basis::ZXY => Basis::YZX,
        }
    }

    #[inline]
    fn to_xyz<T>(self, v: [T; 3]) -> [T; 3] {
        let [m1, m2, m3] = v;
        match self {
            Basis::XYZ => [m1, m2, m3],
            Basis::YZX => [m3, m1, m2],
            Basis::ZXY => [m2, m3, m1],
        }
    }

    #[inline]
    fn index(self) -> usize {
        match self {
            Basis::XYZ => 0,
            Basis::YZX => 1,
            Basis::ZXY => 2,
        }
    }
}

const BASES: [Basis; 3] = [Basis::XYZ, Basis::YZX, Basis::ZXY];

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
enum Face {
    Negative,
    Positive,
}

impl Face {
    #[inline]
    fn to_f32(self) -> f32 {
        match self {
            Face::Negative => -1.0,
            Face::Positive => 1.0,
        }
    }

    fn index(self) -> usize {
        match self {
            Face::Negative => 0,
            Face::Positive => 1,
        }
    }
}

const FACES: [Face; 2] = [Face::Negative, Face::Positive];

#[inline]
fn index_3d(r2: u32, r1: u32, i3: u32, i2: u32, i1: u32) -> u32 {
    (i3 * r2 + i2) * r1 + i1
}

#[inline]
fn index_4d(r3: u32, r2: u32, r1: u32, i4: u32, i3: u32, i2: u32, i1: u32) -> u32 {
    ((i4 * r3 + i3) * r2 + i2) * r1 + i1
}

// Subdivided cube vertex indexing scheme;
// Vertex at (x, y, z) where (0, 0, 0) is (-1, -1, -1).
// Endpoint indexes: (xe, ye, ze) = (x/n, y/n, z/n).
// Center indexes: (xc, yc, zc) = (x - 1, y - 1, z - 1).
// Offset: 0.
// Corners (8): [ze][ye][xe]
// Offset: 8.
// Edges along X (2*2*n): [ze, ye, xc]
// Edges along Y (2*2*n): [xe, ze, yc]
// Edges along Z (2*2*n): [ye, xe, zc]
// Offset: 8 + 12*n.
// Faces with normal X (n*n*2): [zc, yc, xe]
// Faces with normal Y (n*n*2): [xc, zc, ye]
// Faces with normal Z (n*n*2): [yc, xc, ze]
// Length: 8 + 12*n + 6*n*n.

#[inline]
fn corner_index(z: Face, y: Face, x: Face) -> u32 {
    index_3d(2, 2, z.index() as u32, y.index() as u32, x.index() as u32)
}

#[inline]
fn edge_index(n: u32, basis: Basis, i3: Face, i2: Face, i1: u32) -> u32 {
    8 + index_4d(
        2,
        2,
        n,
        basis.index() as u32,
        i3.index() as u32,
        i2.index() as u32,
        i1 - 1,
    )
}

#[inline]
fn face_index(n: u32, basis: Basis, i3: u32, i2: u32, i1: Face) -> u32 {
    8 + 12 * n
        + index_4d(
            n,
            n,
            2,
            basis.index() as u32,
            i3 - 1,
            i2 - 1,
            i1.index() as u32,
        )
}

#[inline]
fn lerp_u32_f32(x: u32, x_range: (u32, u32), y_range: (f32, f32)) -> f32 {
    let (x0, x1) = x_range;
    let (y0, y1) = y_range;
    let (x, x0, x1) = (x as i32, x0 as i32, x1 as i32);
    ((x1 - x) as f32 * y0 + (x - x0) as f32 * y1) / (x1 - x0) as f32
}

#[inline]
unsafe fn to_face(n: u32, i: u32) -> Result<Face, u32> {
    if i == 0 {
        Ok(Face::Negative)
    } else if i < (n + 1) {
        Err(i)
    } else if i == (n + 1) {
        Ok(Face::Positive)
    } else {
        debug_assert!(false, "Index {} out of bounds [0, {}].", i, n + 1);
        std::hint::unreachable_unchecked()
    }
}

// Planes.
#[inline]
unsafe fn plane_index(n: u32, basis: Basis, i3: u32, i2: u32, i1: Face) -> u32 {
    match to_face(n, i3) {
        Ok(i3) => match to_face(n, i2) {
            Ok(i2) => {
                let [x, y, z] = basis.to_xyz([i1, i2, i3]);
                corner_index(z, y, x)
            }
            Err(i2) => edge_index(n, basis.rotate_cw(), i1, i3, i2),
        },
        Err(i3) => match to_face(n, i2) {
            Ok(i2) => edge_index(n, basis.rotate_ccw(), i2, i1, i3),
            Err(i2) => face_index(n, basis, i3, i2, i1),
        },
    }
}

pub fn generate_cube_quads(subdivisions: u32) -> Vec<Quad> {
    // Safe because we promise i3, i2 and i1 are always less than or equal to n + 1.
    unsafe {
        let n = subdivisions;

        let mut faces: Vec<Quad> = Vec::with_capacity((6 * (n + 1) * (n + 1)) as usize);

        for &basis in BASES.into_iter() {
            for &i1 in FACES.into_iter() {
                for i3 in 0..=n {
                    for i2 in 0..=n {
                        faces.push(match i1 {
                            Face::Negative => [
                                plane_index(n, basis, i3, i2, i1),
                                plane_index(n, basis, i3 + 1, i2, i1),
                                plane_index(n, basis, i3 + 1, i2 + 1, i1),
                                plane_index(n, basis, i3, i2 + 1, i1),
                            ],
                            Face::Positive => [
                                plane_index(n, basis, i3, i2, i1),
                                plane_index(n, basis, i3, i2 + 1, i1),
                                plane_index(n, basis, i3 + 1, i2 + 1, i1),
                                plane_index(n, basis, i3 + 1, i2, i1),
                            ],
                        })
                    }
                }
            }
        }

        faces
    }
}

pub fn generate_cube_vertices(radius: f32, subdivisions: u32) -> Vec<[f32; 3]> {
    let n = subdivisions;

    let mut vertices = Vec::with_capacity((8 + 12 * n + 6 * n * n) as usize);

    // Corners vertices.
    for &zf in FACES.into_iter() {
        for &yf in FACES.into_iter() {
            for &xf in FACES.into_iter() {
                debug_assert_eq!(vertices.len(), corner_index(zf, yf, xf) as usize);
                vertices.push([
                    xf.to_f32() * radius,
                    yf.to_f32() * radius,
                    zf.to_f32() * radius,
                ]);
            }
        }
    }

    // Edge vertices.
    for &basis in BASES.into_iter() {
        for &i3 in FACES.into_iter() {
            for &i2 in FACES.into_iter() {
                for i1 in 1..=n {
                    debug_assert_eq!(vertices.len(), edge_index(n, basis, i3, i2, i1) as usize);
                    vertices.push(basis.to_xyz([
                        lerp_u32_f32(i1, (0, n + 1), (-radius, radius)),
                        i2.to_f32() * radius,
                        i3.to_f32() * radius,
                    ]));
                }
            }
        }
    }

    // Face vertices.
    for &basis in BASES.into_iter() {
        for i3 in 1..=n {
            for i2 in 1..=n {
                for &i1 in FACES.into_iter() {
                    debug_assert_eq!(vertices.len(), face_index(n, basis, i3, i2, i1) as usize);
                    vertices.push(basis.to_xyz([
                        i1.to_f32(),
                        lerp_u32_f32(i2, (0, n + 1), (-radius, radius)),
                        lerp_u32_f32(i3, (0, n + 1), (-radius, radius)),
                    ]))
                }
            }
        }
    }

    vertices
}

pub fn generate_cubic_sphere_vertices(radius: f32, n: u32) -> Vec<[f32; 3]> {
    let frac_1_sqrt_3 = f32::sqrt(1.0 / 3.0);
    let frac_acos_frac_1_3_2 = f32::acos(1.0 / 3.0) / 2.0;
    let edge_range = (-frac_acos_frac_1_3_2, frac_acos_frac_1_3_2);
    let face_range = (-FRAC_PI_4, FRAC_PI_4);

    let mut vertices = Vec::with_capacity((8 + 12 * n + 6 * n * n) as usize);

    // Corners vertices.
    for &zf in FACES.into_iter() {
        for &yf in FACES.into_iter() {
            for &xf in FACES.into_iter() {
                debug_assert_eq!(vertices.len(), corner_index(zf, yf, xf) as usize);
                vertices.push([
                    xf.to_f32() * radius * frac_1_sqrt_3,
                    yf.to_f32() * radius * frac_1_sqrt_3,
                    zf.to_f32() * radius * frac_1_sqrt_3,
                ]);
            }
        }
    }

    // Edge vertices.
    for &basis in BASES.into_iter() {
        for &i3 in FACES.into_iter() {
            for &i2 in FACES.into_iter() {
                for i1 in 1..=n {
                    debug_assert_eq!(vertices.len(), edge_index(n, basis, i3, i2, i1) as usize);
                    let a1 = lerp_u32_f32(i1, (0, n + 1), edge_range);
                    vertices.push(basis.to_xyz([
                        f32::sin(a1) * radius,
                        i2.to_f32() * f32::cos(a1) * radius * FRAC_1_SQRT_2,
                        i3.to_f32() * f32::cos(a1) * radius * FRAC_1_SQRT_2,
                    ]));
                }
            }
        }
    }

    // Face vertices.
    for &basis in BASES.into_iter() {
        for i3 in 1..=n {
            for i2 in 1..=n {
                for &i1 in FACES.into_iter() {
                    debug_assert_eq!(vertices.len(), face_index(n, basis, i3, i2, i1) as usize);
                    let a2 = lerp_u32_f32(i2, (0, n + 1), face_range);
                    let a3 = lerp_u32_f32(i3, (0, n + 1), face_range);
                    let ca2 = f32::cos(a2);
                    let sa2 = f32::sin(a2);
                    let ca3 = f32::cos(a3);
                    let sa3 = f32::sin(a3);
                    let s = radius / f32::sqrt(ca2.powi(2) * sa3.powi(2) + ca3.powi(2));
                    vertices.push(basis.to_xyz([
                        s * ca2 * ca3 * i1.to_f32(),
                        s * sa2 * ca3,
                        s * ca2 * sa3,
                    ]))
                }
            }
        }
    }

    debug_assert_eq!(vertices.len(), (8 + 12 * n + 6 * n * n) as usize);

    vertices
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

// pub fn generate_tetra_sphere(
//     scale: f32,
//     subdivisions: u32,
// ) -> (Vec<[f32; 3]>, Vec<Tri>, Vec<u32>) {
//     let mut positions = vec![
//         Vector3::new(0.0, scale, 0.0),
//         Vector3::new(0.0, -(1.0 / 3.0) * scale, f32::sqrt(8.0 / 9.0) * scale),
//         Vector3::new(
//             -f32::sqrt(2.0 / 3.0) * scale,
//             -(1.0 / 3.0) * scale,
//             -f32::sqrt(2.0 / 9.0) * scale,
//         ),
//         Vector3::new(
//             f32::sqrt(2.0 / 3.0) * scale,
//             -(1.0 / 3.0) * scale,
//             -f32::sqrt(2.0 / 9.0) * scale,
//         ),
//     ];

//     // Upper bound.
//     let mut triangles: Vec<Tri> = Vec::with_capacity(4 * 3usize.pow(subdivisions + 1));
//     triangles.extend([[0, 1, 2], [2, 3, 0], [0, 3, 1], [1, 3, 2]].into_iter());

//     let mut objects: Vec<u32> = Vec::with_capacity(subdivisions as usize + 2);
//     objects.extend([0, triangles.len() as u32].into_iter());

//     for subdivision in 0..subdivisions {
//         let triangle_start = objects[subdivision as usize] as usize;
//         let triangle_end = objects[subdivision as usize + 1] as usize;

//         // println!("triangles {}..{}", triangle_start, triangle_end);
//         // for i in triangle_start..triangle_end {
//         //     println!("{:02}: {:?}", i, triangles[i]);
//         // }

//         for t1_idx in triangle_start..triangle_end {
//             let t1 = triangles[t1_idx];
//             let t1_center =
//                 (positions[t1[0] as usize] + positions[t1[1] as usize] + positions[t1[2] as usize])
//                     .normalize_to(scale);

//             let mut poly = Poly::Tri(t1);

//             let mut largest_diff = 0.0;

//             for t2_idx in triangle_start..triangle_end {
//                 if t1_idx == t2_idx {
//                     continue;
//                 }
//                 let t2 = triangles[t2_idx];
//                 let t2_center = (positions[t2[0] as usize]
//                     + positions[t2[1] as usize]
//                     + positions[t2[2] as usize])
//                     .normalize_to(scale);

//                 for i in 0..3 {
//                     let next_i = (i + 1) % 3;
//                     for j in 0..3 {
//                         let next_j = (j + 1) % 3;
//                         if t1[i] == t2[next_j] && t1[next_i] == t2[j] {
//                             let shared_center =
//                                 positions[t1[i] as usize] + positions[t1[next_i] as usize];
//                             let cross_center = t1_center + t2_center;
//                             let diff = cross_center.magnitude2() - shared_center.magnitude2();
//                             if diff > largest_diff {
//                                 largest_diff = diff;
//                                 let prev_i = (i + 2) % 3;
//                                 let prev_j = (j + 2) % 3;
//                                 if t1_idx < t2_idx {
//                                     poly = Poly::Quad([t1[prev_i], t1[i], t2[prev_j], t2[j]]);
//                                 } else {
//                                     poly = Poly::None;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }

//             match poly {
//                 Poly::None => {}
//                 Poly::Tri([a, b, c]) => {
//                     let d = positions.len() as u32;
//                     positions.push(
//                         (positions[a as usize] + positions[b as usize] + positions[c as usize])
//                             .normalize_to(scale),
//                     );
//                     triangles.push([a, b, d]);
//                     triangles.push([b, c, d]);
//                     triangles.push([c, a, d]);
//                 }
//                 Poly::Quad([a, b, c, d]) => {
//                     let e = positions.len() as u32;
//                     positions.push(
//                         (positions[a as usize]
//                             + positions[b as usize]
//                             + positions[c as usize]
//                             + positions[d as usize])
//                             .normalize_to(scale),
//                     );
//                     triangles.push([a, b, e]);
//                     triangles.push([b, c, e]);
//                     triangles.push([c, d, e]);
//                     triangles.push([d, a, e]);
//                 }
//             }
//         }

//         objects.push(triangles.len() as u32);
//     }

//     (positions, triangles, objects)
// }
