use cgmath::*;
use num_traits::{Float, NumCast, ToPrimitive};

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
    /// http://www.geometry.caltech.edu/pubs/UD12.pdf
    #[inline]
    pub fn perspective_infinite_far(self) -> Matrix4<S> {
        // Constants.
        let n2 = S::from(-2.0f64).unwrap();
        let p2 = S::from(2.0f64).unwrap();
        let z = S::zero();

        // Entries.
        let a = self.z1;
        let b = n2 * self.z0 * self.z1;
        let c = p2 * self.z0 * self.z1 / (self.x1 - self.x0);
        let d = p2 * self.z0 * self.z1 / (self.y1 - self.y0);
        let i = (self.x0 + self.x1) / (self.x1 - self.x0);
        let j = (self.y0 + self.y1) / (self.y1 - self.y0);
        let k = self.z1;

        Matrix4::from_cols(
            Vector4::new(c, z, z, z),
            Vector4::new(z, d, z, z),
            Vector4::new(i, j, a, k),
            Vector4::new(z, z, b, z),
        )
    }

    #[inline]
    pub fn orthographic(self) -> Matrix4<S> {
        let p2 = S::from(2.0f64).unwrap();
        let z = S::zero();
        let o = S::one();

        let dx = self.x1 - self.x0;
        let dy = self.y1 - self.y0;
        let dz = self.z1 - self.z0;

        let c = p2 / dx;
        let d = p2 / dy;
        let e = p2 / dz;

        let i = -(self.x0 + self.x1) / dx;
        let j = -(self.y0 + self.y1) / dy;
        let k = -(self.z0 + self.z1) / dz;

        Matrix4::from_cols(
            Vector4::new(c, z, z, z),
            Vector4::new(z, d, z, z),
            Vector4::new(z, z, e, z),
            Vector4::new(i, j, k, o),
        )
    }
}

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
