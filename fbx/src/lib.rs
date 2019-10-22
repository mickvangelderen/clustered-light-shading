#![feature(float_to_from_bytes)]
use std::io;

mod io_ext;
mod num;
mod raw;

use io_ext::*;
use num::*;
pub use raw::*;

pub const MAGIC: [u8; 21] = *b"Kaydara FBX Binary  \0";
pub const VERSION_7300: u32le = u32le::from_ne(7300);

#[derive(Debug, Copy, Clone, Default)]
pub struct FileHeader {
    pub version: u32,
}

impl FileHeader {
    pub fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let header = RawFileHeader::parse(reader)?;
        assert_eq!(header.magic, MAGIC);
        assert_eq!(
            {
                // NOTE: Do an unaligned read of header.version, the pass the copy
                // to assert_eq. It takes a reference by default and reading
                // unaligned values from references is problematic.
                let version = header.version;
                version
            },
            VERSION_7300
        );
        Ok(Self {
            version: header.version.to_ne(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Property {
    Bool(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BoolArray(Vec<u8>),
    I32Array(Vec<i32>),
    I64Array(Vec<i64>),
    F32Array(Vec<f32>),
    F64Array(Vec<f64>),
    String(String),
    Bytes(Vec<u8>),
}

macro_rules! impl_parse_array {
    ($f: ident, $T: ty, $B: ty, $Variant: ident) => {
        fn $f<R: io::Read + io::Seek>(reader: &mut R) -> io::Result<Self> {
            let array_header = RawArrayHeader::parse(reader)?;
            let element_count = array_header.element_count.to_ne() as usize;
            let encoded_byte_count = array_header.byte_count.to_ne() as usize;
            let decoded_byte_count = element_count * std::mem::size_of::<$B>();

            let bytes = match array_header.encoding {
                RawEncodingKind::PLAIN => reader.read_bytes(encoded_byte_count)?,
                RawEncodingKind::DEFLATE => {
                    // NOTE: Because we don't own reader, we can't directly
                    // construct a decoder around it. So we copy into a vec
                    // first and then decode that. Kind of sucks but w/e.
                    let bytes = reader.read_bytes(encoded_byte_count)?;
                    let mut decoder = flate2::bufread::ZlibDecoder::new(&bytes[..]);
                    let mut decoded_bytes = Vec::with_capacity(decoded_byte_count);
                    io::Read::read_to_end(&mut decoder, &mut decoded_bytes).unwrap();
                    decoded_bytes
                }
                unknown => panic!("Unknown encoding: {:?}", unknown),
            };

            assert_eq!(decoded_byte_count, bytes.len());

            // [u8; B*N] -> [[u8; B]; N]
            let bytes = unsafe {
                let mut bytes = std::mem::ManuallyDrop::new(bytes);
                assert_eq!(0, bytes.len() % std::mem::size_of::<$B>());
                assert_eq!(0, bytes.capacity() % std::mem::size_of::<$B>());
                std::vec::Vec::from_raw_parts(
                    bytes.as_mut_ptr() as *mut $B,
                    bytes.len() / std::mem::size_of::<$B>(),
                    bytes.capacity() / std::mem::size_of::<$B>(),
                )
            };

            Ok(Self::$Variant(
                bytes.into_iter().map(<$T>::from_le_bytes).collect(),
            ))
        }
    };
}

impl Property {
    impl_parse_array!(parse_bool_array, u8, [u8; 1], BoolArray);
    impl_parse_array!(parse_i32_array, i32, [u8; 4], I32Array);
    impl_parse_array!(parse_i64_array, i64, [u8; 8], I64Array);
    impl_parse_array!(parse_f32_array, f32, [u8; 4], F32Array);
    impl_parse_array!(parse_f64_array, f64, [u8; 8], F64Array);

    pub fn parse<R: io::Read + io::Seek>(reader: &mut R) -> io::Result<Self> {
        let kind = RawPropertyKind::parse(reader)?;
        Ok(match kind {
            RawPropertyKind::BOOL => {
                let mut value: u8 = 0;
                reader.read_exact(std::slice::from_mut(&mut value))?;
                Self::Bool(value)
            }
            RawPropertyKind::I16 => {
                let mut value: [u8; 2] = Default::default();
                reader.read_exact(&mut value)?;
                Self::I16(i16::from_le_bytes(value))
            }
            RawPropertyKind::I32 => {
                let mut value: [u8; 4] = Default::default();
                reader.read_exact(&mut value)?;
                Self::I32(i32::from_le_bytes(value))
            }
            RawPropertyKind::I64 => {
                let mut value: [u8; 8] = Default::default();
                reader.read_exact(&mut value)?;
                Self::I64(i64::from_le_bytes(value))
            }
            RawPropertyKind::F32 => {
                let mut value: [u8; 4] = Default::default();
                reader.read_exact(&mut value)?;
                Self::F32(f32::from_le_bytes(value))
            }
            RawPropertyKind::F64 => {
                let mut value: [u8; 8] = Default::default();
                reader.read_exact(&mut value)?;
                Self::F64(f64::from_le_bytes(value))
            }
            RawPropertyKind::BOOL_ARRAY => Self::parse_bool_array(reader)?,
            RawPropertyKind::I32_ARRAY => Self::parse_i32_array(reader)?,
            RawPropertyKind::I64_ARRAY => Self::parse_i64_array(reader)?,
            RawPropertyKind::F32_ARRAY => Self::parse_f32_array(reader)?,
            RawPropertyKind::F64_ARRAY => Self::parse_f64_array(reader)?,
            RawPropertyKind::STRING => {
                let byte_count = unsafe { reader.read_val::<u32le>()? }.to_ne() as usize;
                let bytes = reader.read_bytes(byte_count)?;
                Self::String(String::from_utf8(bytes).unwrap())
            }
            RawPropertyKind::BYTES => {
                let byte_count = unsafe { reader.read_val::<u32le>()? }.to_ne() as usize;
                let bytes = reader.read_bytes(byte_count)?;
                Self::Bytes(bytes)
            }
            unknown => {
                panic!("Unknown property: {:?}", unknown.0 as char);
            }
        })
    }

    pub fn to_f32_exact(&self) -> f32 {
        match *self {
            Property::F32(val) => val,
            _ => panic!("Expected f32 but got {:?}", self),
        }
    }

    pub fn to_i32_exact(&self) -> i32 {
        match *self {
            Property::I32(val) => val,
            _ => panic!("Expected i32 but got {:?}", self),
        }
    }

    pub fn to_f64_exact(&self) -> f64 {
        match *self {
            Property::F64(val) => val,
            _ => panic!("Expected f64 but got {:?}", self),
        }
    }

    pub fn to_i64_exact(&self) -> i64 {
        match *self {
            Property::I64(val) => val,
            _ => panic!("Expected i64 but got {:?}", self),
        }
    }

    pub fn as_f32_array_exact(&self) -> &[f32] {
        match *self {
            Property::F32Array(ref val) => val,
            _ => panic!("Expected [f32] but got {:?}", self),
        }
    }

    pub fn as_f64_array_exact(&self) -> &[f64] {
        match *self {
            Property::F64Array(ref val) => val,
            _ => panic!("Expected [f64] but got {:?}", self),
        }
    }

    pub fn as_i32_array_exact(&self) -> &[i32] {
        match *self {
            Property::I32Array(ref val) => val,
            _ => panic!("Expected [i32] but got {:?}", self),
        }
    }

    pub fn as_i64_array_exact(&self) -> &[i64] {
        match *self {
            Property::I64Array(ref val) => val,
            _ => panic!("Expected [i64] but got {:?}", self),
        }
    }

    pub fn as_str(&self) -> &str {
        match *self {
            Property::String(ref val) => val,
            _ => panic!("Expected string but got {:?}", self),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub properties: Vec<Property>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn parse<R: io::Read + io::Seek>(reader: &mut R) -> io::Result<Option<Self>> {
        let header = RawNodeHeader::parse(reader)?;
        let end_offset = header.end_offset.to_ne() as u64;

        match end_offset {
            0 => Ok(None),
            _ => {
                let name = String::from_utf8(reader.read_bytes(header.name_len as usize)?).unwrap();
                let properties = Self::parse_properties(reader, &header)?;
                let children = Self::parse_children(reader, &header)?;

                debug_assert_eq!(end_offset, reader.pos());

                Ok(Some(Node {
                    name,
                    properties,
                    children,
                }))
            }
        }
    }

    #[inline]
    fn parse_properties<R: io::Read + io::Seek>(reader: &mut R, header: &RawNodeHeader) -> io::Result<Vec<Property>> {
        let property_count = header.property_count.to_ne() as usize;
        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            properties.push(Property::parse(reader)?);
        }
        Ok(properties)
    }

    #[inline]
    fn skip_properties<R: io::Read + io::Seek>(reader: &mut R, header: &RawNodeHeader) -> io::Result<()> {
        let properties_byte_count = header.properties_byte_count.to_ne() as i64;
        reader.seek(io::SeekFrom::Current(properties_byte_count))?;
        Ok(())
    }

    #[inline]
    fn parse_children<R: io::Read + io::Seek>(reader: &mut R, header: &RawNodeHeader) -> io::Result<Vec<Node>> {
        let end_offset = header.end_offset.to_ne() as u64;
        let mut children = Vec::new();
        // NOTE(mickvangelderen): Sometimes child nodes aren't "null terminated"
        // so this condition is necessary.
        while reader.pos() < end_offset {
            match Node::parse(reader)? {
                Some(node) => children.push(node),
                None => break,
            }
        }
        Ok(children)
    }
}

pub struct File {
    pub header: FileHeader,
    pub children: Vec<Node>,
}

impl File {
    pub fn parse<R: io::Read + io::Seek>(reader: &mut R) -> io::Result<Self> {
        let header = FileHeader::parse(reader)?;

        let mut children = Vec::new();
        loop {
            match Node::parse(reader)? {
                Some(node) => children.push(node),
                None => break,
            }
        }

        Ok(Self { header, children })
    }
}
