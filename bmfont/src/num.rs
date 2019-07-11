#![allow(non_camel_case_types)]

pub trait Endian {
    type u16: std::fmt::Debug;
    type u32: std::fmt::Debug;
    type u64: std::fmt::Debug;
    type i16: std::fmt::Debug;
    type i32: std::fmt::Debug;
    type i64: std::fmt::Debug;
}

#[derive(Debug)]
pub enum NativeEndian {}

pub type NE = NativeEndian;

#[derive(Debug)]
pub enum LittleEndian {}

pub type LE = LittleEndian;

#[derive(Debug)]
pub enum BigEndian {}

pub type BE = BigEndian;

impl Endian for NativeEndian {
    type u16 = u16;
    type i16 = i16;
    type u32 = u32;
    type i32 = i32;
    type u64 = u64;
    type i64 = i64;
}

impl Endian for LittleEndian {
    type u16 = u16le;
    type i16 = i16le;
    type u32 = u32le;
    type i32 = i32le;
    type u64 = u64le;
    type i64 = i64le;
}

impl Endian for BigEndian {
    type u16 = u16be;
    type i16 = i16be;
    type u32 = u32be;
    type i32 = i32be;
    type u64 = u64be;
    type i64 = i64be;
}

macro_rules! impl_ints {
    ($BE:ident, $LE:ident, $N:ident, $B:ty) => {
        impl_ints!(@ $BE, $N, $B, to_be_bytes, from_be_bytes);
        impl_ints!(@ $LE, $N, $B, to_le_bytes, from_le_bytes);
    };
    (@ $TE:ident, $N:ident, $B:ty, $into:ident, $from:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct $TE($B);

        impl $TE {
            #[inline]
            pub const fn from_bytes(val: $B) -> Self {
                $TE(val)
            }

            #[inline]
            pub const fn into_bytes(self) -> $B {
                self.0
            }

            #[inline]
            pub const fn to_bytes(&self) -> $B {
                self.0
            }

            #[inline]
            pub const fn from_ne(val: $N) -> Self {
                $TE($N::$into(val))
            }

            #[inline]
            pub const fn into_ne(self) -> $N {
                $N::$from(self.0)
            }

            #[inline]
            pub const fn to_ne(&self) -> $N {
                $N::$from(self.0)
            }
        }

        impl From<$B> for $TE {
            #[inline]
            fn from(val: $B) -> Self {
                $TE::from_bytes(val)
            }
        }

        impl From<$TE> for $B {
            #[inline]
            fn from(val: $TE) -> Self {
                $TE::to_bytes(&val)
            }
        }

        impl From<$N> for $TE {
            #[inline]
            fn from(val: $N) -> Self {
                $TE::from_ne(val)
            }
        }

        impl From<$TE> for $N {
            #[inline]
            fn from(val: $TE) -> Self {
                $TE::to_ne(&val)
            }
        }

        impl std::fmt::Debug for $TE {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, concat!(stringify!($TE), "({})"), self.to_ne())
            }
        }
    };
}

impl_ints!(u16be, u16le, u16, [u8; 2]);
impl_ints!(i16be, i16le, i16, [u8; 2]);
impl_ints!(u32be, u32le, u32, [u8; 4]);
impl_ints!(i32be, i32le, i32, [u8; 4]);
impl_ints!(u64be, u64le, u64, [u8; 8]);
impl_ints!(i64be, i64le, i64, [u8; 8]);
