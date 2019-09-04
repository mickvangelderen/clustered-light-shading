#![allow(unused_macros)]

#[macro_use]
mod frame_pool;

macro_rules! get_uniform_location {
    ($gl: ident, $program: expr, $s: expr) => {{
        let loc = $gl.get_uniform_location($program, gl::static_cstr!($s));
        if loc.is_none() {
            warn!("Could not get uniform location {:?}.", $s);
        }
        loc
    }};
}

macro_rules! get_attribute_location {
    ($gl: ident, $program: expr, $s: expr) => {{
        let loc = $gl.get_attrib_location($program, gl::static_cstr!($s));
        if loc.is_none() {
            warn!("Could not get attribute location {:?}.", $s);
        }
        loc
    }};
}

macro_rules! glsl_defines {
    ($fn: ident {
        bindings: {
            $($bname: ident = $bval: expr;)*
        },
        uniforms: {
            $($uname: ident = $uval: expr;)*
        },
    }) => {
        $(
            pub const $bname: u32 = $bval;
        )*

            $(
                pub const $uname: gl::UniformLocation = unsafe { gl::UniformLocation::from_i32_unchecked($uval) };
            )*

        fn $fn() -> String {
            let mut s = String::new();
            $(
                s.push_str(&format!("#define {} {}\n", stringify!($bname), $bname));
            )*
                $(
                    s.push_str(&format!("#define {} {}\n", stringify!($uname), $uname.to_i32()));
                )*
                s
        }
    };
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
            pub fn new<F>(mut f: F) -> Self where F: FnMut($Key) -> T {
                $Map {
                    $(
                        $field: f($Key::$variant),
                    )*
                }
            }

            #[inline]
            pub fn map<U, F>(self, mut f: F) -> $Map<U> where F: FnMut(T) -> U {
                $Map {
                    $(
                        $field: f(self.$field),
                    )*
                }
            }

            #[inline]
            pub fn zip<U, V, F>(self, other: $Map<U>, mut f: F) -> $Map<V> where F: FnMut(T, U) -> V {
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

macro_rules! create_framebuffer {
    ($gl: expr, ($ds_att: expr, $ds_tex: expr), $( ($col_att: expr, $col_tex: expr) ),* $(,)?) => {{
        let framebuffer_name = $gl.create_framebuffer();

        $(
            $gl.named_framebuffer_texture(framebuffer_name, $col_att, $col_tex, 0);
        )*

        $gl.named_framebuffer_texture(framebuffer_name, $ds_att, $ds_tex, 0);

        $gl.named_framebuffer_draw_buffers(
            framebuffer_name,
            &[$( $col_att.into() ),*],
        );

        assert_eq!(
            $gl.check_named_framebuffer_status(framebuffer_name, gl::FRAMEBUFFER),
            gl::FRAMEBUFFER_COMPLETE.into()
        );

        framebuffer_name
    }}
}

macro_rules! rendering_context {
    ($object: ident) => {
        crate::rendering::RenderingContext {
            gl: &$object.gl,
            resource_dir: &$object.resource_dir,
            current: &mut $object.current,
            shader_compiler: &mut $object.shader_compiler,
        }
    };
}

macro_rules! shader_compilation_context {
    ($object: ident) => {
        crate::shader_compiler::ShaderCompilationContext {
            resource_dir: &$object.resource_dir,
            current: &mut $object.current,
            shader_compiler: &mut $object.shader_compiler,
        }
    };
}
