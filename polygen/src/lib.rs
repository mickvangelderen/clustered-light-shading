// NOTE: A lot of things are private to this module because I want to retain
// flexibility in the implementations.

use cgmath::*;

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
        match self {
            Basis::XYZ => {
                let [x, y, z] = v;
                [x, y, z]
            }
            Basis::YZX => {
                let [y, z, x] = v;
                [x, y, z]
            }
            Basis::ZXY => {
                let [z, x, y] = v;
                [x, y, z]
            }
        }
    }

    fn from_xyz<T>(self, v: [T; 3]) -> [T; 3] {
        let [x, y, z] = v;
        match self {
            Basis::XYZ => [x, y, z],
            Basis::YZX => [y, z, x],
            Basis::ZXY => [z, x, y],
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

    fn select<T>(self, range: (T, T)) -> T {
        match self {
            Face::Negative => range.0,
            Face::Positive => range.1,
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
    8 + 12 * n + index_4d(n, n, 2, basis.index() as u32, i3 - 1, i2 - 1, i1.index() as u32)
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

pub fn quad_vertices(x: (f32, f32), y: (f32, f32), z: (f32, f32), subdivisions: u32) -> Vec<[f32; 3]> {
    let n = subdivisions;

    let mut vertices = Vec::with_capacity((8 + 12 * n + 6 * n * n) as usize);

    // Corners vertices.
    for &zf in FACES.into_iter() {
        for &yf in FACES.into_iter() {
            for &xf in FACES.into_iter() {
                debug_assert_eq!(vertices.len(), corner_index(zf, yf, xf) as usize);
                vertices.push([xf.select(x), yf.select(y), zf.select(z)]);
            }
        }
    }

    // Edge vertices.
    for &basis in BASES.into_iter() {
        let [r1, r2, r3] = basis.from_xyz([x, y, z]);
        for &i3 in FACES.into_iter() {
            for &i2 in FACES.into_iter() {
                for i1 in 1..=n {
                    debug_assert_eq!(vertices.len(), edge_index(n, basis, i3, i2, i1) as usize);
                    vertices.push(basis.to_xyz([lerp_u32_f32(i1, (0, n + 1), r1), i2.select(r2), i3.select(r3)]));
                }
            }
        }
    }

    // Face vertices.
    for &basis in BASES.into_iter() {
        let [r1, r2, r3] = basis.from_xyz([x, y, z]);
        for i3 in 1..=n {
            for i2 in 1..=n {
                for &i1 in FACES.into_iter() {
                    debug_assert_eq!(vertices.len(), face_index(n, basis, i3, i2, i1) as usize);
                    vertices.push(basis.to_xyz([
                        i1.select(r1),
                        lerp_u32_f32(i2, (0, n + 1), r2),
                        lerp_u32_f32(i3, (0, n + 1), r3),
                    ]))
                }
            }
        }
    }

    vertices
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
                        faces.extend_from_slice(
                            P::from_quad(match i1 {
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
                            .as_slice(),
                        );
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

pub fn cube_vertices(x: (f32, f32), y: (f32, f32), z: (f32, f32), subdivisions: u32) -> Vec<[f32; 3]> {
    let n = subdivisions;

    let mut vertices = Vec::with_capacity((8 + 12 * n + 6 * n * n) as usize);

    // Corners vertices.
    for &zf in FACES.into_iter() {
        for &yf in FACES.into_iter() {
            for &xf in FACES.into_iter() {
                debug_assert_eq!(vertices.len(), corner_index(zf, yf, xf) as usize);
                vertices.push([xf.select(x), yf.select(y), zf.select(z)]);
            }
        }
    }

    // Edge vertices.
    for &basis in BASES.into_iter() {
        let [r1, r2, r3] = basis.from_xyz([x, y, z]);
        for &i3 in FACES.into_iter() {
            for &i2 in FACES.into_iter() {
                for i1 in 1..=n {
                    debug_assert_eq!(vertices.len(), edge_index(n, basis, i3, i2, i1) as usize);
                    vertices.push(basis.to_xyz([lerp_u32_f32(i1, (0, n + 1), r1), i2.select(r2), i3.select(r3)]));
                }
            }
        }
    }

    // Face vertices.
    for &basis in BASES.into_iter() {
        let [r1, r2, r3] = basis.from_xyz([x, y, z]);
        for i3 in 1..=n {
            for i2 in 1..=n {
                for &i1 in FACES.into_iter() {
                    debug_assert_eq!(vertices.len(), face_index(n, basis, i3, i2, i1) as usize);
                    vertices.push(basis.to_xyz([
                        i1.select(r1),
                        lerp_u32_f32(i2, (0, n + 1), r2),
                        lerp_u32_f32(i3, (0, n + 1), r3),
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
                    vertices.push(basis.to_xyz([s * sa1, s * ca1 * i2.to_f32(), s * ca1 * i3.to_f32()]))
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
                    vertices.push(basis.to_xyz([s * ca2 * ca3 * i1.to_f32(), s * sa2 * ca3, s * ca2 * sa3]))
                }
            }
        }
    }

    debug_assert_eq!(vertices.len(), (8 + 12 * n + 6 * n * n) as usize);

    vertices
}

pub fn compute_normals(triangles: &[Tri], pos_in_obj: &[[f32; 3]]) -> Vec<[f32; 3]> {
    // Compute normal directions for each triangle.
    let unnormalized_normals_per_triangle: Vec<Vector3<f32>> = triangles
        .iter()
        .map(|&tri| {
            let p0 = Vector3::from(pos_in_obj[tri[0] as usize]);
            let p1 = Vector3::from(pos_in_obj[tri[1] as usize]);
            let p2 = Vector3::from(pos_in_obj[tri[2] as usize]);
            ((p1 - p0).cross(p2 - p0))
        })
        .collect();

    // At this point we know all triangle position indices are less than pos_in_obj.len();

    let mut vertex_to_triangles: Vec<Vec<usize>> = (0..pos_in_obj.len())
        .into_iter()
        .map(|_| Vec::with_capacity(8)) // I think 6 should be the average of a nice tri mesh.
        .collect();

    for (tri_idx, &tri) in triangles.iter().enumerate() {
        unsafe {
            vertex_to_triangles.get_unchecked_mut(tri[0] as usize).push(tri_idx);
            vertex_to_triangles.get_unchecked_mut(tri[1] as usize).push(tri_idx);
            vertex_to_triangles.get_unchecked_mut(tri[2] as usize).push(tri_idx);
        }
    }

    // For each vertex, sum the normal directions of all triangles containing it, and normalize.
    vertex_to_triangles
        .into_iter()
        .map(|tri_idxs: Vec<usize>| {
            tri_idxs
                .into_iter()
                .fold(Vector3::zero(), |sum, tri_idx: usize| unsafe {
                    sum + *unnormalized_normals_per_triangle.get_unchecked(tri_idx)
                })
                .normalize()
                .into()
        })
        .collect()
}

pub fn compute_tangents(triangles: &[Tri], pos_in_obj: &[[f32; 3]], pos_in_tex: &[[f32; 2]]) -> Vec<[f32; 3]> {
    // Compute tangents per triangle.
    let unnormalized_tangents_per_triangle: Vec<Vector3<f32>> = triangles
        .iter()
        .map(|&tri| {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let p0 = Vector3::from(pos_in_obj[i0]);
            let p1 = Vector3::from(pos_in_obj[i1]);
            let p2 = Vector3::from(pos_in_obj[i2]);

            // TODO: Think of the proper way to handle this. Should meshes be
            // allowed to have partially specified texture coordinates?
            if i0 < pos_in_tex.len() && i1 < pos_in_tex.len() && i2 < pos_in_tex.len() {
                let t0 = Vector2::from(pos_in_tex[i0]);
                let t1 = Vector2::from(pos_in_tex[i1]);
                let t2 = Vector2::from(pos_in_tex[i2]);

                (t1[1] - t2[1]) * p0 + (t2[1] - t0[1]) * p1 + (t0[1] - t1[1]) * p2
            } else {
                Vector3::zero()
            }
        })
        .collect();

    // At this point we know all triangle position indices are less than pos_in_obj.len();

    let mut vertex_to_triangles: Vec<Vec<usize>> = (0..pos_in_obj.len())
        .into_iter()
        .map(|_| Vec::with_capacity(8)) // I think 6 should be the average of a nice tri mesh.
        .collect();

    for (tri_idx, &tri) in triangles.iter().enumerate() {
        unsafe {
            vertex_to_triangles.get_unchecked_mut(tri[0] as usize).push(tri_idx);
            vertex_to_triangles.get_unchecked_mut(tri[1] as usize).push(tri_idx);
            vertex_to_triangles.get_unchecked_mut(tri[2] as usize).push(tri_idx);
        }
    }

    // For each vertex, sum the normal directions of all triangles containing it, and normalize.
    vertex_to_triangles
        .into_iter()
        .map(|tri_idxs: Vec<usize>| {
            tri_idxs
                .into_iter()
                .fold(Vector3::zero(), |sum, tri_idx: usize| unsafe {
                    sum + *unnormalized_tangents_per_triangle.get_unchecked(tri_idx)
                })
                .normalize()
                .into()
        })
        .collect()
}
