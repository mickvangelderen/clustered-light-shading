use cgmath::Matrix4;
use cgmath::Vector4;

pub trait FromHmd<T> {
    fn from_hmd(val: T) -> Self;
}

pub trait HmdInto<T> {
    fn hmd_into(self) -> T;
}

impl<T, U> HmdInto<U> for T
where
    U: FromHmd<T>,
{
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

pub trait Flatten<T> {
    fn flatten(self) -> T;
}

pub trait Unflatten<T> {
    fn unflatten(self) -> T;
}

macro_rules! impl_flatten_unflatten {
    ($($N: expr,)*) => {
        $(
            // Immutable slices.
            impl<'a, T: 'a> Flatten<&'a [T]> for &'a [[T; $N]] {
                fn flatten(self) -> &'a [T] {
                    unsafe {
                        std::slice::from_raw_parts(
                            self.as_ptr() as *const T,
                            self.len() * $N,
                        )
                    }
                }
            }

            impl<'a, T: 'a> Unflatten<&'a [[T; $N]]> for &'a [T] {
                fn unflatten(self) -> &'a [[T; $N]] {
                    unsafe {
                        debug_assert!(self.len() % $N == 0);
                        std::slice::from_raw_parts(
                            self.as_ptr() as *const [T; $N],
                            self.len() / $N,
                        )
                    }
                }
            }

            // Mutable slices.
            impl<'a, T: 'a> Flatten<&'a mut [T]> for &'a mut [[T; $N]] {
                fn flatten(self) -> &'a mut [T] {
                    unsafe {
                        std::slice::from_raw_parts_mut(
                            self.as_mut_ptr() as *mut T,
                            self.len() * $N,
                        )
                    }
                }
            }

            impl<'a, T: 'a> Unflatten<&'a mut [[T; $N]]> for &'a mut [T] {
                fn unflatten(self) -> &'a mut [[T; $N]] {
                    unsafe {
                        debug_assert!(self.len() % $N == 0);
                        std::slice::from_raw_parts_mut(
                            self.as_mut_ptr() as *mut [T; $N],
                            self.len() / $N,
                        )
                    }
                }
            }

            // Vec.
            impl<T> Flatten<Vec<T>> for Vec<[T; $N]> {
                fn flatten(mut self) -> Vec<T> {
                    unsafe {
                        let ptr = self.as_mut_ptr();
                        let len = self.len();
                        let cap = self.capacity();
                        std::mem::forget(self);
                        Vec::from_raw_parts(ptr as *mut T, len * $N, cap)
                    }
                }
            }

            impl<T> Unflatten<Vec<[T; $N]>> for Vec<T> {
                fn unflatten(mut self) -> Vec<[T; $N]> {
                    unsafe {
                        debug_assert!(self.len() % $N == 0);
                        let ptr = self.as_mut_ptr();
                        let len = self.len();
                        let cap = self.capacity();
                        std::mem::forget(self);
                        Vec::from_raw_parts(ptr as *mut [T; $N], len / $N, cap)
                    }
                }
            }
        )*
    };
}

impl_flatten_unflatten!(1, 2, 3, 4,);
