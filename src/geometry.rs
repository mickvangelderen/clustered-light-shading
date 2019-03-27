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
// ny0
// 0yn
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
    fn vector(self, m1: f32, m2: f32, m3: f32) -> Vector3<f32> {
        match self {
            Basis::XYZ => Vector3::new(m1, m2, m3),
            Basis::YZX => Vector3::new(m3, m1, m2),
            Basis::ZXY => Vector3::new(m2, m3, m1),
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

// Corners are indexed by (face, face, face)
#[inline]
fn corner_index(z: Face, y: Face, x: Face) -> u32 {
    index_3d(2, 2, z.index() as u32, y.index() as u32, x.index() as u32)
}

// Edges are indexed by (face, face, n)
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

// Faces are indexed by (n, n, face)
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
fn plane_index(n: u32, basis: Basis, i3: u32, i2: u32, i1: Face) -> u32 {
    match to_face(n, i3) {
        Ok(i3) => match to_face(n, i2) {
            Ok(i2) => {
                let [z, y, x] = match basis {
                    Basis::XYZ => [i3, i2, i1],
                    Basis::YZX => [i2, i1, i3],
                    Basis::ZXY => [i1, i3, i2],
                };
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

pub fn generate_cubic_sphere(
    radius: f32,
    subdivisions: u32,
) -> (Vec<Vector3<f32>>, Vec<Quad>, Vec<u32>) {
    let sqrt_frac_1_3 = f32::sqrt(1.0 / 3.0);
    let acos_frac_1_3 = f32::acos(1.0 / 3.0);

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

    // Edges.
    for &basis in BASES.into_iter() {
        for &i3 in FACES.into_iter() {
            let m3 = i3.to_f32();
            for &i2 in FACES.into_iter() {
                let m2 = i2.to_f32();
                for i1 in 1..=n {
                    let angle = index_to_f32(n, i1) * acos_frac_1_3 / 2.0;
                    debug_assert_eq!(positions.len(), edge_index(n, basis, i3, i2, i1) as usize);
                    positions.push(basis.vector(
                        radius * f32::sin(angle),
                        m2 * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                        m3 * std::f32::consts::FRAC_1_SQRT_2 * radius * f32::cos(angle),
                    ));
                }
            }
        }
    }

    // Faces.
    for &basis in BASES.into_iter() {
        for i3 in 1..=n {
            let beta = index_to_f32(n, i3) * std::f32::consts::FRAC_PI_4;
            for i2 in 1..=n {
                let alpha = index_to_f32(n, i2) * std::f32::consts::FRAC_PI_4;
                for &i1 in FACES.into_iter() {
                    debug_assert_eq!(positions.len(), face_index(n, basis, i3, i2, i1) as usize);
                    positions.push(basis.vector(
                        i1.to_f32() * radius * f32::cos(beta) * f32::cos(alpha),
                        radius * f32::cos(beta) * f32::sin(alpha),
                        radius * f32::sin(beta),
                    ));
                }
            }
        }
    }

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
