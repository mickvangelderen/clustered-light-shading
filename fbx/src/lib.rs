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
        // NOTE: Unaligned read.
        let version = header.version;
        assert_eq!(version, VERSION_7300);
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
            let byte_count = array_header.byte_count.to_ne() as usize;
            match array_header.encoding {
                RawEncodingKind::PLAIN => {
                    assert_eq!(element_count * std::mem::size_of::<$B>(), byte_count);
                }
                RawEncodingKind::DEFLATE => {
                    // Nothing to assert.
                }
                unknown => panic!("Unknown encoding: {:?}", unknown),
            };

            let bytes = unsafe { reader.read_vec::<u8>(byte_count)? };

            let bytes = match array_header.encoding {
                RawEncodingKind::PLAIN => {
                    // Nothing to do.
                    bytes
                }
                RawEncodingKind::DEFLATE => {
                    let mut decoder = flate2::read::ZlibDecoder::new(&bytes[..]);
                    let mut decoded_bytes = Vec::with_capacity(element_count * std::mem::size_of::<$B>());
                    match io::Read::read_to_end(&mut decoder, &mut decoded_bytes) {
                        Ok(_) => {
                            assert_eq!(element_count * std::mem::size_of::<$B>(), decoded_bytes.len());
                        }
                        Err(err) => {
                            eprintln!("Decoding error: {:?}", err);
                        }
                    }
                    decoded_bytes
                }
                unknown => panic!("Unknown encoding: {:?}", unknown),
            };

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
                let bytes = unsafe { reader.read_vec::<u8>(byte_count)? };
                Self::String(String::from_utf8(bytes).unwrap())
            }
            RawPropertyKind::BYTES => {
                let byte_count = unsafe { reader.read_val::<u32le>()? }.to_ne() as usize;
                let bytes = unsafe { reader.read_vec::<u8>(byte_count)? };
                Self::Bytes(bytes)
            }
            unknown => {
                panic!("Unknown property: {:?}", unknown.0 as char);
            }
        })
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

        if end_offset == 0 {
            return Ok(None);
        }

        let name = String::from_utf8(unsafe { reader.read_vec::<u8>(header.name_len as usize)? }).unwrap();

        let prop_count = header.property_count.to_ne();
        let mut properties = Vec::with_capacity(prop_count as usize);
        for _ in 0..prop_count {
            properties.push(Property::parse(reader)?);
        }

        let mut children = Vec::new();

        while reader.seek(io::SeekFrom::Current(0)).unwrap() < end_offset {
            match Node::parse(reader)? {
                Some(node) => children.push(node),
                None => break,
            }
        }

        assert_eq!(end_offset, reader.seek(io::SeekFrom::Current(0)).unwrap());

        Ok(Some(Node {
            name,
            properties,
            children,
        }))
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
            let node = Node::parse(reader)?;
            match node {
                Some(node) => children.push(node),
                None => break,
            }
        }

        Ok(Self { header, children })
    }
}
