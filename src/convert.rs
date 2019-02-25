use cgmath::Matrix4;
use cgmath::Vector4;

pub trait FromHmd<T> {
    fn from_hmd(val: T) -> Self;
}

pub trait HmdInto<T> {
    fn hmd_into(self) -> T;
}

impl<T, U> HmdInto<U> for T where U: FromHmd<T> {
    #[inline]
    fn hmd_into(self) -> U {
        U::from_hmd(self)
    }
}

impl FromHmd<[[f32; 4]; 4]> for Matrix4<f32> {
    #[inline]
    fn from_hmd(m: [[f32; 4]; 4]) -> Self {
        Matrix4::from_cols(
            Vector4::new(m[0][0], m[1][0], m[2][0], m[3][0]),
            Vector4::new(m[0][1], m[1][1], m[2][1], m[3][1]),
            Vector4::new(m[0][2], m[1][2], m[2][2], m[3][2]),
            Vector4::new(m[0][3], m[1][3], m[2][3], m[3][3]),
        )
    }
}

impl FromHmd<[[f32; 4]; 3]> for Matrix4<f32> {
    #[inline]
    fn from_hmd(m: [[f32; 4]; 3]) -> Self {
        Matrix4::from_cols(
            Vector4::new(m[0][0], m[1][0], m[2][0], 0.0),
            Vector4::new(m[0][1], m[1][1], m[2][1], 0.0),
            Vector4::new(m[0][2], m[1][2], m[2][2], 0.0),
            Vector4::new(m[0][3], m[1][3], m[2][3], 1.0),
        )
    }
}
