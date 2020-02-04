use cgmath::{Matrix4, Vector3};
use num_traits::Float;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Plane3<T> {
    pub normal: Vector3<T>,
    pub distance: T,
}

impl<T> Plane3<T>
where
    T: Float,
{
    pub fn reflection_matrix(&self) -> Matrix4<T> {
        let z0 = T::zero();
        let p1 = T::one();
        let n2 = -(p1 + p1);
        let nx = n2 * self.normal.x;
        let ny = n2 * self.normal.y;
        let nz = n2 * self.normal.z;
        Matrix4::new(
            // Column 0
            p1 + nx * self.normal.x,
            nx * self.normal.y,
            nx * self.normal.z,
            z0,
            // Column 1
            ny * self.normal.x,
            p1 + ny * self.normal.y,
            ny * self.normal.z,
            z0,
            // Column 2
            nz * self.normal.x,
            nz * self.normal.y,
            p1 + nz * self.normal.z,
            z0,
            // Column 3
            -nx * self.distance,
            -ny * self.distance,
            -nz * self.distance,
            p1,
        )
    }
}
