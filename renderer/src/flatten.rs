#![warn(clippy::modulo_one)]

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
