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

        // Intermediates.
        let dx = self.x1 - self.x0;
        let dy = self.y1 - self.y0;

        // Entries.
        let a = self.z1;
        let b = n2 * self.z0 * self.z1;
        let c = p2 * self.z0 * self.z1 / dx;
        let d = p2 * self.z0 * self.z1 / dy;
        let i = (self.x0 + self.x1) / dx;
        let j = (self.y0 + self.y1) / dy;
        let k = self.z1;

        Matrix4::from_cols(
            Vector4::new(c, z, z, z),
            Vector4::new(z, d, z, z),
            Vector4::new(i, j, a, k),
            Vector4::new(z, z, b, z),
        )
    }

    #[inline]
    pub fn perspective(self) -> Matrix4<S> {
        // Constants.
        let n2 = S::from(-2.0f64).unwrap();
        let p2 = S::from(2.0f64).unwrap();
        let z = S::zero();

        // Entries.
        let a = n2 * self.z0 / self.dx();
        let b = (self.x0 + self.x1) / self.dx();
        let c = n2 * self.z0 / self.dy();
        let d = (self.y0 + self.y1) / self.dy();
        let e = -(self.z0 + self.z1) / self.dz();
        let f = p2 * self.z0 * self.z1 / self.dz();
        let g = -S::one();

        Matrix4::from_cols(
            Vector4::new(a, z, z, z),
            Vector4::new(z, c, z, z),
            Vector4::new(b, d, e, g),
            Vector4::new(z, z, f, z),
        )
    }

    #[inline]
    pub fn perspective_z0p1(self) -> Matrix4<S> {
        // Constants.
        let n2 = S::from(-2.0f64).unwrap();
        let z = S::zero();

        // Entries.
        let a = n2 * self.z0 / self.dx();
        let b = (self.x0 + self.x1) / self.dx();
        let c = n2 * self.z0 / self.dy();
        let d = (self.y0 + self.y1) / self.dy();
        let e = -self.z1 / self.dz();
        let f = self.z0 * self.z1 / self.dz();
        let g = -S::one();

        Matrix4::from_cols(
            Vector4::new(a, z, z, z),
            Vector4::new(z, c, z, z),
            Vector4::new(b, d, e, g),
            Vector4::new(z, z, f, z),
        )
    }

    #[inline]
    pub fn orthographic(self) -> Matrix4<S> {
        let p2 = S::from(2.0f64).unwrap();
        let z = S::zero();
        let o = S::one();

        let c = p2 / self.dx();
        let d = p2 / self.dy();
        let e = p2 / self.dz();

        let i = -(self.x0 + self.x1) / self.dx();
        let j = -(self.y0 + self.y1) / self.dy();
        let k = -(self.z0 + self.z1) / self.dz();

        Matrix4::from_cols(
            Vector4::new(c, z, z, z),
            Vector4::new(z, d, z, z),
            Vector4::new(z, z, e, z),
            Vector4::new(i, j, k, o),
        )
    }

    #[inline]
    fn dx(self) -> S {
        self.x1 - self.x0
    }

    #[inline]
    fn dy(self) -> S {
        self.y1 - self.y0
    }

    #[inline]
    fn dz(self) -> S {
        self.z1 - self.z0
    }

    pub fn line_mesh(self) -> ([[S; 3]; 8], [[u32; 2]; 12]) {
        let Frustrum { x0, x1, y0, y1, z0, z1 } = self;
        let vertices = [
            [x0, y0, z0],
            [x1, y0, z0],
            [x0, y1, z0],
            [x1, y1, z0],
            [x0, y0, z1],
            [x1, y0, z1],
            [x0, y1, z1],
            [x1, y1, z1],
        ];
        let indices = [
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
        (vertices, indices)
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
