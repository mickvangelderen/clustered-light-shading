#![allow(unused_macros)]

macro_rules! get_uniform_location {
    ($gl: ident, $program: expr, $s: expr) => {{
        let loc = $gl.get_uniform_location($program, gl::static_cstr!($s));
        if loc.is_none() {
            eprintln!("{}: Could not get uniform location {:?}.", file!(), $s);
        }
        loc
    }};
}

macro_rules! get_attribute_location {
    ($gl: ident, $program: expr, $s: expr) => {{
        let loc = $gl.get_attrib_location($program, gl::static_cstr!($s));
        if loc.is_none() {
            eprintln!("{}: Could not get attribute location {:?}.", file!(), $s);
        }
        loc
    }};
}

macro_rules! field_offset {
    ($Struct:ty, $field:ident) => {
        &(*(std::ptr::null::<$Struct>())).$field as *const _ as usize
    };
}

macro_rules! impl_enum_map {
    (
        $Key: ident => struct $Map: ident { $(
            $variant: ident => $field: ident,
        )* }
    ) => {
        #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $Map<T> {
            $(
                pub $field: T,
            )*
        }

        impl<T> $Map<T> {
            #[inline]
            pub fn new<F>(f: F) -> Self where F: Fn($Key) -> T {
                $Map {
                    $(
                        $field: f($Key::$variant),
                    )*
                }
            }

            #[inline]
            pub fn map<U, F>(self, f: F) -> $Map<U> where F: Fn(T) -> U {
                $Map {
                    $(
                        $field: f(self.$field),
                    )*
                }
            }

            #[inline]
            pub fn zip<U, V, F>(self, other: $Map<U>, f: F) -> $Map<V> where F: Fn(T, U) -> V {
                $Map {
                    $(
                        $field: f(self.$field, other.$field),
                    )*
                }
            }

            #[inline]
            pub fn as_ref(&self) -> $Map<&T> {
                $Map {
                    $(
                        $field: &self.$field,
                    )*
                }
            }

            #[inline]
            pub fn as_mut(&mut self) -> $Map<&mut T> {
                $Map {
                    $(
                        $field: &mut self.$field,
                    )*
                }
            }
        }

        impl<T> std::ops::Index<$Key> for $Map<T> {
            type Output = T;

            #[inline]
            fn index<'a>(&'a self, key: $Key) -> &'a Self::Output {
                match key {
                    $(
                        $Key::$variant => &self.$field,
                    )*
                }
            }
        }

        impl<T> std::ops::IndexMut<$Key> for $Map<T> {
            #[inline]
            fn index_mut<'a>(&'a mut self, key: $Key) -> &'a mut Self::Output {
                match key {
                    $(
                        $Key::$variant => &mut self.$field,
                    )*
                }
            }
        }
    };
}

macro_rules! impl_enum_and_enum_map {
    (
        $(#[$($key_meta: tt)*])?
        enum $Key: ident => struct $Map: ident { $(
            $variant: ident => $field: ident,
        )* }
    ) => {
        $(#[$($key_meta)*])?
        pub enum $Key {
            $(
                $variant,
            )*
        }

        impl_enum_map!(
            $Key => struct $Map { $(
                $variant => $field,
            )* }
        );
    }
}
