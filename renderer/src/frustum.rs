use super::range::Range3;
use cgmath::*;
use num_traits::{cast, Float};

/// linear interpolation coefficients
fn lic<T: Float>(x0: T, x1: T, y0: T, y1: T) -> (T, T) {
    let d = T::one() / (x1 - x0);
    ((y1 - y0) * d, (x1 * y0 - x0 * y1) * d)
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
    pub x0: T,
    pub x1: T,
    pub y0: T,
    pub y1: T,
    pub z0: T,
    pub z1: T,
}

impl<T: Float> Frustum<T> {
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

    #[inline]
    pub fn orthographic(&self, range: &Range3<T>) -> Matrix4<T> {
        let zero = T::zero();
        let one = T::one();
        let (ax, bx) = lic(self.x0, self.x1, range.x0, range.x1);
        let (ay, by) = lic(self.y0, self.y1, range.y0, range.y1);
        let (az, bz) = lic(self.z0, self.z1, range.z0, range.z1);
        Matrix4::new(
            ax, zero, zero, zero, // c0
            zero, ay, zero, zero, // c1
            zero, zero, az, zero, // c2
            bx, by, bz, one, // c3
        )
    }

    #[inline]
    pub fn inverse_orthographic(&self, range: &Range3<T>) -> Matrix4<T> {
        let zero = T::zero();
        let one = T::one();
        let (ax, bx) = lic(range.x0, range.x1, self.x0, self.x1);
        let (ay, by) = lic(range.y0, range.y1, self.y0, self.y1);
        let (az, bz) = lic(range.z0, range.z1, self.z0, self.z1);
        Matrix4::new(
            ax, zero, zero, zero, // c0
            zero, ay, zero, zero, // c1
            zero, zero, az, zero, // c2
            bx, by, bz, one, // c3
        )
    }

    #[inline]
    pub fn perspective(&self, range: &Range3<T>) -> Matrix4<T> {
        let zero = T::zero();
        let one = T::one();
        let (ax, bx) = lic(self.x0, self.x1, range.x0, range.x1);
        let (ay, by) = lic(self.y0, self.y1, range.y0, range.y1);
        let (az, bz) = {
            let d = one / (self.z1 - self.z0);
            (
                (range.z1 - range.z0) * (self.z0 * self.z1) * d,
                (self.z1 * range.z1 - self.z0 * range.z0) * d,
            )
        };

        Matrix4::new(
            ax, zero, zero, zero, // c0
            zero, ay, zero, zero, // c1
            -bx, -by, -bz, -one, // c2
            zero, zero, az, zero, // c3
        )
    }

    #[inline]
    pub fn inverse_perspective(&self, range: &Range3<T>) -> Matrix4<T> {
        let zero = T::zero();
        let one = T::one();
        let (ax, bx) = lic(range.x0, range.x1, self.x0, self.x1);
        let (ay, by) = lic(range.y0, range.y1, self.y0, self.y1);
        let (az, bz) = {
            let d = one / ((range.z1 - range.z0) * self.z0 * self.z1);
            ((self.z1 - self.z0) * d, (range.z0 * self.z0 - range.z1 * self.z1) * d)
        };

        Matrix4::new(
            ax, zero, zero, zero, // c0
            zero, ay, zero, zero, // c1
            zero, zero, zero, az, // c2
            bx, by, -one, bz, // c3
        )
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
    pub fn perspective_vertices(&self) -> [Point3<T>; 8] {
        [
            Point3::new(-self.z0 * self.x0, -self.z0 * self.y0, self.z0),
            Point3::new(-self.z0 * self.x1, -self.z0 * self.y0, self.z0),
            Point3::new(-self.z0 * self.x0, -self.z0 * self.y1, self.z0),
            Point3::new(-self.z0 * self.x1, -self.z0 * self.y1, self.z0),
            Point3::new(-self.z1 * self.x0, -self.z1 * self.y0, self.z1),
            Point3::new(-self.z1 * self.x1, -self.z1 * self.y0, self.z1),
            Point3::new(-self.z1 * self.x0, -self.z1 * self.y1, self.z1),
            Point3::new(-self.z1 * self.x1, -self.z1 * self.y1, self.z1),
        ]
    }

    #[inline]
    pub fn orthographic_vertices(&self) -> [Point3<T>; 8] {
        [
            Point3::new(self.x0, self.y0, self.z0),
            Point3::new(self.x1, self.y0, self.z0),
            Point3::new(self.x0, self.y1, self.z0),
            Point3::new(self.x1, self.y1, self.z0),
            Point3::new(self.x0, self.y0, self.z1),
            Point3::new(self.x1, self.y0, self.z1),
            Point3::new(self.x0, self.y1, self.z1),
            Point3::new(self.x1, self.y1, self.z1),
        ]
    }

    #[inline]
    pub fn line_mesh_indices(&self) -> [[u32; 2]; 12] {
        [
            // Back
            [0b000, 0b001],
            [0b001, 0b011],
            [0b011, 0b010],
            [0b010, 0b000],
            // Front
            [0b100, 0b101],
            [0b101, 0b111],
            [0b111, 0b110],
            [0b110, 0b100],
            // Side
            [0b000, 0b100],
            [0b001, 0b101],
            [0b010, 0b110],
            [0b011, 0b111],
        ]
    }

    #[inline]
    pub fn cast<U>(self) -> Option<Frustum<U>>
    where
        U: Float,
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    const RANGE: Range3<f64> = Range3 {
        x0: -1.0,
        x1: 1.0,
        y0: -1.0,
        y1: 1.0,
        z0: -1.0,
        z1: 1.0,
    };

    #[test]
    fn inverses() {
        let f = Frustum {
            x0: -0.1,
            x1: 0.2,
            y0: -0.3,
            y1: 0.4,
            z0: -0.5,
            z1: -0.6,
        };

        assert_relative_eq!(
            Matrix4::identity(),
            f.orthographic(&RANGE) * f.inverse_orthographic(&RANGE)
        );
        assert_relative_eq!(
            Matrix4::identity(),
            f.perspective(&RANGE) * f.inverse_perspective(&RANGE)
        );
    }
}
