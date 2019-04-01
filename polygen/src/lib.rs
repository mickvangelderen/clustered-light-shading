// NOTE: A lot of things are private to this module because I want to retain
// flexibility in the implementations.

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

trait Array<T>: Sized {
    const LENGTH: usize;

    fn as_slice(&self) -> &[T];
}

macro_rules! impl_arrays {
    ($($N: expr,)*) => {
        $(
            impl<T> Array<T> for [T; $N] {
                const LENGTH: usize = $N;

                fn as_slice(&self) -> &[T] {
                    self
                }
            }
        )*
    }
}

impl_arrays!(1, 2,);

trait FromQuad<T>: Sized {
    type Polygons: Array<Self>;

    fn from_quad(vertices: [T; 4]) -> Self::Polygons;
}

impl<T> FromQuad<T> for [T; 4] {
    type Polygons = [Self; 1];

    fn from_quad(quad: [T; 4]) -> Self::Polygons {
        [quad]
    }
}

impl<T: Copy> FromQuad<T> for [T; 3] {
    type Polygons = [Self; 2];

    fn from_quad(quad: [T; 4]) -> Self::Polygons {
        let [a, b, c, d] = quad;
        [[a, b, c], [c, d, a]]
    }
}


fn cube_polygons<P: FromQuad<u32> + Clone>(subdivisions: u32) -> Vec<P> {
    // Safe because we promise i3, i2 and i1 are always less than or equal to n + 1.
    unsafe {
        let n = subdivisions;

        let mut faces: Vec<P> = Vec::with_capacity(P::Polygons::LENGTH * (6 * (n + 1) * (n + 1)) as usize);

        for &basis in BASES.into_iter() {
            for &i1 in FACES.into_iter() {
                for i3 in 0..=n {
                    for i2 in 0..=n {
                        faces.extend_from_slice(P::from_quad(match i1 {
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
                        }).as_slice());
                    }
                }
            }
        }

        faces
    }
}

pub fn cube_tris(subdivisions: u32) -> Vec<Tri> {
    cube_polygons(subdivisions)
}

pub fn cube_quads(subdivisions: u32) -> Vec<Quad> {
    cube_polygons(subdivisions)
}

pub fn cube_vertices(radius: f32, subdivisions: u32) -> Vec<[f32; 3]> {
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

pub fn cubic_sphere_vertices(radius: f32, subdivisions: u32) -> Vec<[f32; 3]> {
    let n = subdivisions;
    let frac_1_sqrt_3 = f32::sqrt(1.0 / 3.0);
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
                    let a1 = lerp_u32_f32(i1, (0, n + 1), face_range);
                    let ca1 = f32::cos(a1);
                    let sa1 = f32::sin(a1);
                    let s = radius / f32::sqrt(1.0 + ca1.powi(2));
                    vertices.push(basis.to_xyz([
                        s * sa1,
                        s * ca1 * i2.to_f32(),
                        s * ca1 * i3.to_f32(),
                    ]))
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
