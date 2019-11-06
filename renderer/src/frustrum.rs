use cgmath::*;
use num_traits::{Float, NumCast, ToPrimitive, cast};

unsafe fn reinterpret<A, B>(a: &A) -> &B {
    assert_eq!(std::mem::size_of::<A>(), std::mem::size_of::<B>());
    &*(a as *const A as *const B)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Frustrum<S> {
    pub x0: S,
    pub x1: S,
    pub y0: S,
    pub y1: S,
    pub z0: S,
    pub z1: S,
}

impl AsRef<[f32; 6]> for Frustrum<f32> {
    #[inline]
    fn as_ref(&self) -> &[f32; 6] {
        unsafe { reinterpret(self) }
    }
}

impl<S: Float> Frustrum<S> {
    #[inline]
    pub fn corners_in_clp(depth_range: (S, S)) -> [Point3<S>; 8] {
        let (x0, y0, x1, y1) = (-S::one(), -S::one(), S::one(), S::one());
        let (z0, z1) = depth_range;
        [
            Point3::new(x0, y0, z0),
            Point3::new(x1, y0, z0),
            Point3::new(x0, y1, z0),
            Point3::new(x1, y1, z0),
            Point3::new(x0, y0, z1),
            Point3::new(x1, y0, z1),
            Point3::new(x0, y1, z1),
            Point3::new(x1, y1, z1),
        ]
    }

    #[inline]
    pub fn perspective(self, depth_range: (S, S)) -> Matrix4<S> {
        // Constants.
        let zero = S::zero();
        let one = S::one();
        let two = one + one;

        // Parameters.
        let Frustrum { x0, x1, y0, y1, z0, z1 } = self;
        let (r0, r1) = depth_range;

        // Intermediates.
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dz = z1 - z0;
        let dr = r1 - r0;

        let sx = x0 + x1;
        let sy = y0 + y1;

        Matrix4::from_cols(
            Vector4::new(
                //
                -two * z0 / dx,
                zero,
                zero,
                zero,
            ),
            Vector4::new(
                //
                zero,
                -two * z0 / dy,
                zero,
                zero,
            ),
            Vector4::new(
                //
                sx / dx,
                sy / dy,
                -(r1 * z1 - r0 * z0) / dz,
                -one,
            ),
            Vector4::new(
                //
                zero,
                zero,
                (z0 * z1 * dr) / dz,
                zero,
            ),
        )
    }

    /// http://www.geometry.caltech.edu/pubs/UD12.pdf
    #[inline]
    pub fn perspective_infinite_far(self, depth_range: (S, S)) -> Matrix4<S> {
        // Constants.
        let zero = S::zero();
        let one = S::one();
        let two = one + one;

        // Parameters.
        let Frustrum { x0, x1, y0, y1, z0, .. } = self;
        let (r0, r1) = depth_range;

        // Intermediates.
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dr = r1 - r0;

        let sx = x0 + x1;
        let sy = y0 + y1;

        Matrix4::from_cols(
            Vector4::new(
                //
                -two * z0 / dx,
                zero,
                zero,
                zero,
            ),
            Vector4::new(
                //
                zero,
                -two * z0 / dy,
                zero,
                zero,
            ),
            Vector4::new(
                //
                sx / dx,
                sy / dy,
                -r1,
                -one,
            ),
            Vector4::new(
                //
                zero,
                zero,
                z0 * dr,
                zero,
            ),
        )
    }

    #[inline]
    pub fn orthographic(self, depth_range: (S, S)) -> Matrix4<S> {
        // Constants.
        let zero = S::zero();
        let one = S::one();
        let two = one + one;

        // Parameters.
        let Frustrum { x0, x1, y0, y1, z0, z1 } = self;
        let (r0, r1) = depth_range;

        // Intermediates.
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dz = z1 - z0;
        let dr = r1 - r0;

        let sx = x0 + x1;
        let sy = y0 + y1;

        Matrix4::from_cols(
            Vector4::new(
                //
                two / dx,
                zero,
                zero,
                zero,
            ),
            Vector4::new(
                //
                zero,
                two / dy,
                zero,
                zero,
            ),
            Vector4::new(
                //
                zero,
                zero,
                dr / dz,
                zero,
            ),
            Vector4::new(
                //
                -sx / dx,
                -sy / dy,
                (r0 * z1 - r1 * z0) / dz,
                one,
            ),
        )
    }
}

pub static FRUSTRUM_LINE_MESH_INDICES: &[[u32; 2]; 12] = &[
    // Front
    [0, 1],
    [2, 3],
    [0, 2],
    [1, 3],
    // Back
    [4, 5],
    [6, 7],
    [4, 6],
    [5, 7],
    // Side
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

impl<S: ToPrimitive> Frustrum<S> {
    /// Component-wise casting to another type
    #[inline]
    pub fn cast<T: NumCast>(self) -> Option<Frustrum<T>> {
        Some(Frustrum {
            x0: T::from(self.x0)?,
            x1: T::from(self.x1)?,
            y0: T::from(self.y0)?,
            y1: T::from(self.y1)?,
            z0: T::from(self.z0)?,
            z1: T::from(self.z1)?,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ClassicFrustum<T> {
    pub l: T,
    pub r: T,
    pub b: T,
    pub t: T,
    pub n: T,
    pub f: T,
}

#[derive(Debug, Copy, Clone)]
pub struct Frustum<T> {
    /// -l/n
    pub x0: T,
    /// r/n
    pub x1: T,
    /// -b/n
    pub y0: T,
    /// t/n
    pub y1: T,
    /// -f
    pub z0: T,
    /// -n
    pub z1: T,
}

struct Coefficients<T> {
    pub a_x: T,
    pub a_y: T,
    pub a_z: T,
    pub b_x: T,
    pub b_y: T,
    pub b_z: T,
}

impl<T> Frustum<T>
where
    T: Float,
{
    #[inline]
    pub fn zero() -> Self {
        Self {
            x0: T::zero(),
            x1: T::zero(),
            y0: T::zero(),
            y1: T::zero(),
            z0: T::zero(),
            z1: T::zero(),
        }
    }

    #[inline]
    pub fn from_classic(classic: &ClassicFrustum<T>) -> Self {
        Self {
            x0: classic.l / classic.n,
            x1: classic.r / classic.n,
            y0: classic.b / classic.n,
            y1: classic.t / classic.n,
            z0: -classic.f,
            z1: -classic.n,
        }
    }

    #[inline]
    pub fn from_range(range: &Range3<T>) -> Self {
        let Range3 { x0, x1, y0, y1, z0, z1 } = *range;
        Self { x0, x1, y0, y1, z0, z1 }
    }

    /// Returns a matrix that takes [x_cam, y_cam, z_cam, 1] and turns it into [-z*x_cls, -z*y_cls, z_cls, -z].
    #[inline]
    pub fn cluster_perspective(&self, range: &Range3<T>) -> Matrix4<T> {
        let Coefficients {
            a_x,
            a_y,
            a_z,
            b_x,
            b_y,
            b_z,
        } = self.coefficients(range);

        let o_o = T::zero();

        Matrix4::from_cols(
            Vector4::new(a_x, o_o, o_o, o_o),
            Vector4::new(o_o, a_y, o_o, o_o),
            Vector4::new(-b_x, -b_y, a_z, -T::one()),
            Vector4::new(o_o, o_o, b_z, o_o),
        )
    }

    #[inline]
    pub fn cluster_orthographic(&self, range: &Range3<T>) -> Matrix4<T> {
        let Coefficients {
            a_x,
            a_y,
            a_z,
            b_x,
            b_y,
            b_z,
        } = self.coefficients(range);

        let o_o = T::zero();

        Matrix4::from_cols(
            Vector4::new(a_x, o_o, o_o, o_o),
            Vector4::new(o_o, a_y, o_o, o_o),
            Vector4::new(o_o, o_o, a_z, o_o),
            Vector4::new(b_x, b_y, b_z, T::one()),
        )
    }

    #[inline]
    pub fn corners_in_cam_perspective(&self) -> [Point3<T>; 8] {
        let Self { x0, x1, y0, y1, z0, z1 } = *self;
        [
            Point3::new(-z0*x0, -z0*y0, z0),
            Point3::new(-z0*x1, -z0*y0, z0),
            Point3::new(-z0*x0, -z0*y1, z0),
            Point3::new(-z0*x1, -z0*y1, z0),
            Point3::new(-z1*x0, -z1*y0, z1),
            Point3::new(-z1*x1, -z1*y0, z1),
            Point3::new(-z1*x0, -z1*y1, z1),
            Point3::new(-z1*x1, -z1*y1, z1),
        ]
    }

    #[inline]
    pub fn corners_in_cam_orthographic(&self) -> [Point3<T>; 8] {
        let Self { x0, x1, y0, y1, z0, z1 } = *self;
        [
            Point3::new(x0, y0, z0),
            Point3::new(x1, y0, z0),
            Point3::new(x0, y1, z0),
            Point3::new(x1, y1, z0),
            Point3::new(x0, y0, z1),
            Point3::new(x1, y0, z1),
            Point3::new(x0, y1, z1),
            Point3::new(x1, y1, z1),
        ]
    }

    #[inline]
    fn coefficients(&self, range: &Range3<T>) -> Coefficients<T> {
        Coefficients {
            a_x: range.dx() / self.dx(),
            a_y: range.dy() / self.dy(),
            a_z: range.dz() / self.dz(),
            b_x: (range.x0 * self.x1 - range.x1 * self.x0) / self.dx(),
            b_y: (range.y0 * self.y1 - range.y1 * self.y0) / self.dy(),
            b_z: (range.z0 * self.z1 - range.z1 * self.z0) / self.dz(),
        }
    }

    #[inline]
    pub fn dx(&self) -> T {
        self.x1 - self.x0
    }

    #[inline]
    pub fn dy(&self) -> T {
        self.y1 - self.y0
    }

    #[inline]
    pub fn dz(&self) -> T {
        self.z1 - self.z0
    }

    #[inline]
    pub fn cast<U>(self) -> Option<Frustum<U>> where U: Float {
        Some(Frustum {
            x0: cast(self.x0)?,
            x1: cast(self.x1)?,
            y0: cast(self.y0)?,
            y1: cast(self.y1)?,
            z0: cast(self.z0)?,
            z1: cast(self.z1)?,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Range3<T> {
    pub x0: T,
    pub x1: T,
    pub y0: T,
    pub y1: T,
    pub z0: T,
    pub z1: T,
}

impl<T> Range3<T>
where
    T: Float,
{
    #[inline]
    pub fn from_vector(v: Vector3<T>) -> Self {
        Self {
            x0: T::zero(),
            x1: v.x,
            y0: T::zero(),
            y1: v.y,
            z0: T::zero(),
            z1: v.z,
        }
    }

    #[inline]
    pub fn from_point(p: Point3<T>) -> Self {
        Self {
            x0: p.x,
            x1: p.x,
            y0: p.y,
            y1: p.y,
            z0: p.z,
            z1: p.z,
        }
    }

    #[inline]
    pub fn from_points<I>(points: I) -> Option<Self>
    where
        I: IntoIterator<Item = Point3<T>>,
    {
        let mut points = points.into_iter();
        if let Some(first) = points.next() {
            let mut range = Self::from_point(first);
            for point in points {
                range = range.include_point(point);
            }
            Some(range)
        } else {
            None
        }
    }

    #[inline]
    pub fn include_point(self, p: Point3<T>) -> Self {
        Range3 {
            x0: self.x0.min(p.x),
            x1: self.x1.max(p.x),
            y0: self.y0.min(p.y),
            y1: self.y1.max(p.y),
            z0: self.z0.min(p.z),
            z1: self.z1.max(p.z),
        }
    }

    #[inline]
    pub fn dx(&self) -> T {
        self.x1 - self.x0
    }

    #[inline]
    pub fn dy(&self) -> T {
        self.y1 - self.y0
    }

    #[inline]
    pub fn dz(&self) -> T {
        self.z1 - self.z0
    }

    #[inline]
    pub fn min(&self) -> Point3<T> {
        Point3::new(self.x0, self.y0, self.z0)
    }

    #[inline]
    pub fn max(&self) -> Point3<T> {
        Point3::new(self.x1, self.y1, self.z1)
    }

    #[inline]
    pub fn center(&self) -> Point3<T> {
        let two: T = cast(2.0).unwrap();
        Point3::new(
            (self.x0 + self.x1) / two,
            (self.y0 + self.y1) / two,
            (self.z0 + self.z1) / two,
        )
    }

    #[inline]
    pub fn delta(&self) -> Vector3<T> {
        // NOTE: Would use `self.max() - self.min()` but cgmath uses BaseNum
        // and it will affect everything depending on it so we just write
        // the code ourselves.
        Vector3::new(self.x1 - self.x0, self.y1 - self.y0, self.z1 - self.z0)
    }

    #[inline]
    pub fn cast<U>(self) -> Option<Range3<U>> where U: Float {
        Some(Range3 {
            x0: cast(self.x0)?,
            x1: cast(self.x1)?,
            y0: cast(self.y0)?,
            y1: cast(self.y1)?,
            z0: cast(self.z0)?,
            z1: cast(self.z1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_invert_cluster_projection() {
        let dimensions = Vector3::new(5, 4, 3);

        let frustum = {
            let w_over_h = 4.0 / 3.0;
            let l = -1.0;
            let b = l / w_over_h;

            Frustum {
                x0: l,
                x1: -l,
                y0: b,
                y1: -b,
                z0: -20.0,
                z1: -1.0,
            }
        };

        let cam_to_clp = frustum.cluster_perspective(&Range3::from_vector(dimensions.cast().unwrap()));

        println!("{:#?}", &cam_to_clp);

        let pairs = [
            (Point3::new(0.0, 0.0, -1.0), Point3::new(2.5, 2.0, 3.0)),
            (Point3::new(0.0, 0.0, -10.5), Point3::new(2.5, 2.0, 1.5)),
            (Point3::new(0.0, 0.0, -20.0), Point3::new(2.5, 2.0, 0.0)),
        ];

        for &(pos_in_cam, expected_pos_in_cls) in &pairs {
            let p = cam_to_clp * pos_in_cam.to_homogeneous();
            let pos_in_cls = Point3::new(p.x / p.w, p.y / p.w, p.z);

            assert_relative_eq!(expected_pos_in_cls, pos_in_cls);
        }

        let clp_to_cam = cam_to_clp.invert().unwrap();

        println!("{:#?}", &clp_to_cam);

        for &(expected_pos_in_cam, pos_in_cls) in &pairs {
            let a_z = cam_to_clp[2][2];
            let b_z = cam_to_clp[3][2];
            let neg_z_cam = (b_z - pos_in_cls.z) / a_z;
            let p = clp_to_cam
                * Vector4::new(
                    neg_z_cam * pos_in_cls.x,
                    neg_z_cam * pos_in_cls.y,
                    pos_in_cls.z,
                    neg_z_cam,
                );
            let pos_in_cam = Point3::new(p.x / p.w, p.y / p.w, p.z);

            assert_relative_eq!(expected_pos_in_cam, pos_in_cam, epsilon = 20.0 * std::f64::EPSILON);
        }
    }
}
