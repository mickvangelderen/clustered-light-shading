use crate::convert::*;
use crate::num::*;
use std::io;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RawPropertyKind(pub u8);

impl RawPropertyKind {
    pub const BOOL: Self = Self(b'C');
    pub const I16: Self = Self(b'Y');
    pub const I32: Self = Self(b'I');
    pub const I64: Self = Self(b'L');
    pub const F32: Self = Self(b'F');
    pub const F64: Self = Self(b'D');
    pub const BOOL_ARRAY: Self = Self(b'b');
    pub const I32_ARRAY: Self = Self(b'i');
    pub const I64_ARRAY: Self = Self(b'l');
    pub const F32_ARRAY: Self = Self(b'f');
    pub const F64_ARRAY: Self = Self(b'd');
    pub const STRING: Self = Self(b'S');
    pub const BYTES: Self = Self(b'R');

    pub fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = 0u8;
        reader.read_exact(std::slice::from_mut(&mut value))?;
        Ok(Self(value))
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct RawEncodingKind(pub u32le);

impl RawEncodingKind {
    pub const PLAIN: Self = Self(u32le::from_bytes([0, 0, 0, 0]));
    pub const DEFLATE: Self = Self(u32le::from_bytes([1, 0, 0, 0]));
}

macro_rules! impl_parse {
    ($T: ident) => {
        impl $T {
            pub fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
                unsafe {
                    let mut value = std::mem::MaybeUninit::<Self>::uninit();
                    reader.read_exact(std::slice::from_raw_parts_mut(
                        value.as_mut_ptr() as *mut u8,
                        std::mem::size_of::<Self>(),
                    ))?;
                    Ok(value.assume_init())
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C, packed)]
pub struct RawFileHeader {
    pub magic: [u8; 21],
    pub unknown: [u8; 2],
    pub version: u32le,
}

impl_parse!(RawFileHeader);

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct RawArrayHeader {
    pub element_count: u32le,
    pub encoding: RawEncodingKind,
    pub byte_count: u32le,
}

impl_parse!(RawArrayHeader);

#[derive(Debug, Copy, Clone, Default)]
#[repr(C, packed)]
pub struct RawNodeHeader {
    pub end_offset: u32le,
    pub property_count: u32le,
    pub properties_byte_count: u32le,
    pub name_len: u8,
}

impl_parse!(RawNodeHeader);
