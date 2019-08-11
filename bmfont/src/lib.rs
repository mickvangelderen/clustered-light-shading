#![feature(const_int_conversion)]

//! http://www.angelcode.com/products/bmfont/doc/file_format.html

mod bmfont;
mod num;

pub(crate) use std::convert::{TryFrom, TryInto};
pub(crate) use std::ffi::CStr;

pub use bmfont::*;
pub use num::*;

/// Promise this type can be reinterpreted from a byte slice of the right size.
pub unsafe trait Raw {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockKind {
    Info,
    Common,
    Pages,
    Chars,
    KerningPairs,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InvalidBlockKind(u8);

impl TryFrom<u8> for BlockKind {
    type Error = InvalidBlockKind;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(BlockKind::Info),
            2 => Ok(BlockKind::Common),
            3 => Ok(BlockKind::Pages),
            4 => Ok(BlockKind::Chars),
            5 => Ok(BlockKind::KerningPairs),
            invalid => Err(InvalidBlockKind(invalid)),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FileHeader {
    pub magic: [u8; 3],
    pub version: u8,
}

unsafe impl Raw for FileHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct BlockHeader {
    pub kind: u8,
    pub byte_size: u32le,
}

unsafe impl Raw for BlockHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct InfoBlock {
    pub font_size: u16le,
    pub bit_field: u8,
    // std::int8_t reserved:4;
    // std::int8_t bold:1;
    // std::int8_t italic:1;
    // std::int8_t unicode:1;
    // std::int8_t smooth:1;
    pub char_set: u8,
    pub stretch_y: u16le,
    pub super_sampling_level: i8,
    pub padding_py: u8,
    pub padding_px: u8,
    pub padding_ny: u8,
    pub padding_nx: u8,
    pub spacing_x: u8,
    pub spacing_y: u8,
    pub outline: u8,
}

unsafe impl Raw for InfoBlock {}

#[derive(Debug)]
#[repr(C)]
pub struct CommonBlock {
    pub line_y: u16le,
    pub base: u16le,
    pub scale_x: u16le,
    pub scale_y: u16le,
    pub pages: u16le,
    pub bit_field: u8,
    // std::uint8_t packed:1;
    // std::uint8_t reserved:7;
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

unsafe impl Raw for CommonBlock {}

#[derive(Debug)]
#[repr(C)]
pub struct CharBlock {
    pub id: u32le,
    pub x: u16le,
    pub y: u16le,
    pub width: u16le,
    pub height: u16le,
    pub offset_x: i16le,
    pub offset_y: i16le,
    pub advance_x: i16le,
    pub page: i8,
    pub channel: i8,
}

unsafe impl Raw for CharBlock {}

#[derive(Debug)]
#[repr(C)]
pub struct KerningPairBlock {
    pub first: u32le,
    pub second: u32le,
    pub amount: i16le,
}

unsafe impl Raw for KerningPairBlock {}

pub struct Input<'a>(&'a [u8]);

impl<'a> Input<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub fn read_raw<T>(&mut self) -> Option<&'a T>
    where
        T: Raw,
    {
        let byte_size = std::mem::size_of::<T>();

        if self.0.len() < byte_size {
            None
        } else {
            let output = unsafe { &*(self.0.as_ptr() as *const T) };
            self.0 = &self.0[byte_size..];
            Some(output)
        }
    }

    pub fn read_raw_array<T>(&mut self, count: usize) -> Option<&'a [T]>
    where
        T: Raw,
    {
        let byte_size = std::mem::size_of::<T>() * count;

        if self.0.len() < byte_size {
            None
        } else {
            let output = unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const T, count) };
            self.0 = &self.0[byte_size..];
            Some(output)
        }
    }

    pub fn read_bytes(&mut self, count: usize) -> Option<&'a [u8]> {
        if self.0.len() < count {
            None
        } else {
            let output = &self.0[0..count];
            self.0 = &self.0[count..];
            Some(output)
        }
    }

    pub fn bytes(&self) -> &'a [u8] {
        self.0
    }
}
