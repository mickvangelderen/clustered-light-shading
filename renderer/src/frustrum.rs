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
    #[inline]
    pub fn perspective(self, depth_range: (S, S)) -> Matrix4<S> {
        // Constants.
        let zero = S::zero();
        let one = S::one();
        let two = one + one;

        // Parameters.
        let Frustrum {
            x0,
            x1,
            y0,
            y1,
            z0,
            z1,
        } = self;
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
        let Frustrum {
            x0, x1, y0, y1, z0, ..
        } = self;
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
        let Frustrum {
            x0,
            x1,
            y0,
            y1,
            z0,
            z1,
        } = self;
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

    pub fn line_mesh(self) -> ([[S; 3]; 8], [[u32; 2]; 12]) {
        let Frustrum {
            x0,
            x1,
            y0,
            y1,
            z0,
            z1,
        } = self;
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
