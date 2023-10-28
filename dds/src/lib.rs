// Somewhat helpful: https://docs.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression

use belene::*;
use std::io;

pub mod bc1;
pub mod bc2;
pub mod bc3;
pub mod color;

/// Pixel information as represented in the DDS file
///
/// Direct translation of struct found here:
/// https://msdn.microsoft.com/en-us/library/bb943984.aspx
#[repr(C)]
#[derive(Debug)]
pub struct RawPixelFormat {
    pub size: u32le,
    pub flags: u32le,
    pub four_cc: [u8; 4],
    pub rgb_bit_count: u32le,
    pub red_bit_mask: u32le,
    pub green_bit_mask: u32le,
    pub blue_bit_mask: u32le,
    pub alpha_bit_mask: u32le,
}

/// Header as represented in the DDS file
///
/// Direct translation of struct found here:
/// https://msdn.microsoft.com/en-us/library/bb943982.aspx
#[repr(C)]
#[derive(Debug)]
pub struct RawFileHeader {
    pub magic: [u8; 4],
    pub size: u32le,
    pub flags: u32le,
    pub height: u32le,
    pub width: u32le,
    pub pitch_or_linear_size: u32le,
    pub depth: u32le,
    pub mipmap_count: u32le,
    pub _reserved_0: [u32le; 11],
    pub pixel_format: RawPixelFormat,
    pub caps0: Caps0,
    pub caps1: Caps1,
    pub caps2: u32le,
    pub caps3: u32le,
    pub _reserved_1: u32le,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Caps0(pub u32le);

impl Caps0 {
    const COMPLEX: u32le = u32le([0x08, 0x00, 0x00, 0x00]);
    const TEXTURE: u32le = u32le([0x00, 0x10, 0x00, 0x00]);
    const MIPMAP: u32le = u32le([0x00, 0x00, 0x40, 0x00]);

    pub fn is_complex(&self) -> bool {
        let v = self.0.to_ne();
        let m = Self::COMPLEX.to_ne();
        v & m == m
    }

    pub fn is_texture(&self) -> bool {
        let v = self.0.to_ne();
        let m = Self::TEXTURE.to_ne();
        v & m == m
    }

    pub fn is_mipmap(&self) -> bool {
        let v = self.0.to_ne();
        let m = Self::MIPMAP.to_ne();
        v & m == m
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Caps1(pub u32le);

impl Caps1 {
    const CUBEMAP: u32le = u32le([0x00, 0x02, 0x00, 0x00]);
    // static const uint32_t kCaps2CubeMapPosXMask = 0x400;
    // static const uint32_t kCaps2CubeMapNegXMask = 0x800;
    // static const uint32_t kCaps2CubeMapPosYMask = 0x1000;
    // static const uint32_t kCaps2CubeMapNegYMask = 0x2000;
    // static const uint32_t kCaps2CubeMapPosZMask = 0x4000;
    // static const uint32_t kCaps2CubeMapNegZMask = 0x8000;
    // static const uint32_t kCaps2VolumeMask = 0x200000;

    pub fn is_cubemap(&self) -> bool {
        let v = self.0.to_ne();
        let m = Self::CUBEMAP.to_ne();
        v & m == m
    }
}

impl RawFileHeader {
    pub fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        unsafe {
            let mut buffer = std::mem::MaybeUninit::<Self>::uninit();
            reader.read_exact(std::slice::from_raw_parts_mut(
                buffer.as_mut_ptr() as *mut u8,
                std::mem::size_of::<Self>(),
            ))?;
            Ok(buffer.assume_init())
        }
    }
}

#[derive(Debug)]
pub struct FileHeader {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mipmap_count: u32,
    pub is_cubemap: bool,
    pub pixel_format: Format,
}

impl From<RawFileHeader> for FileHeader {
    fn from(header: RawFileHeader) -> Self {
        let mipmap_count = header.mipmap_count.to_ne();
        let width = header.width.to_ne();
        let height = header.height.to_ne();
        let depth = header.depth.to_ne();
        let is_cubemap = header.caps1.is_cubemap();
        let pixel_format_flags = header.pixel_format.flags.to_ne();

        let pixel_format: Format = if pixel_format_flags & PixelFormatFlags::FOURCC == PixelFormatFlags::FOURCC {
            match header.pixel_format.four_cc {
                fourcc::DXT1 => Format::BC1_UNORM_RGB,
                fourcc::DXT2 | fourcc::DXT3 => Format::BC2_UNORM_RGBA,
                fourcc::DXT4 | fourcc::DXT5 => Format::BC3_UNORM_RGBA,
                fourcc::ATI1 | fourcc::BC4U => Format::BC4_UNORM_R,
                fourcc::BC4S => Format::BC4_SNORM_R,
                fourcc::ATI2 | fourcc::BC5U => Format::BC5_UNORM_RG,
                fourcc::BC5S => Format::BC5_SNORM_RG,
                other => {
                    panic!(
                        "Unsupported four_cc: {:?} in header {:#?}.",
                        std::str::from_utf8(&other).unwrap(),
                        &header
                    );
                }
            }
        } else {
            panic!(
                "Unsupported pixel format flags: {:?} in header {:#?}.",
                pixel_format_flags, &header
            );
        };

        Self {
            width,
            height,
            depth,
            mipmap_count,
            is_cubemap,
            pixel_format,
        }
    }
}

#[derive(Debug)]
pub struct Layer {
    pub byte_offset: usize,
    pub byte_count: usize,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct File {
    pub header: FileHeader,
    pub layers: Vec<Layer>,
    pub bytes: Vec<u8>,
}

pub mod fourcc {
    pub const DXT1: [u8; 4] = *b"DXT1";
    pub const DXT2: [u8; 4] = *b"DXT2";
    pub const DXT3: [u8; 4] = *b"DXT3";
    pub const DXT4: [u8; 4] = *b"DXT4";
    pub const DXT5: [u8; 4] = *b"DXT5";
    pub const ATI1: [u8; 4] = *b"ATI1";
    pub const BC4U: [u8; 4] = *b"BC4U";
    pub const BC4S: [u8; 4] = *b"BC4S";
    pub const ATI2: [u8; 4] = *b"ATI2";
    pub const BC5U: [u8; 4] = *b"BC5U";
    pub const BC5S: [u8; 4] = *b"BC5S";
    pub const RGBG: [u8; 4] = *b"RGBG";
    pub const GRGB: [u8; 4] = *b"GRGB";
    pub const YUY2: [u8; 4] = *b"YUY2";
}

pub struct PixelFormatFlags;

impl PixelFormatFlags {
    pub const ALPHAPIXELS: u32 = 0x1;
    pub const ALPHA: u32 = 0x2;
    pub const FOURCC: u32 = 0x4;
    pub const RGB: u32 = 0x40;
    pub const YUV: u32 = 0x200;
    pub const LUMINANCE: u32 = 0x20000;
    pub const BUMP: u32 = 0x80000; // From falcor.
}

pub enum ComponentType {
    FLOAT,
    UNORM,
    SNORM,
    UINT,
    SINT,
}

macro_rules! impl_format {
    ($(
        ($Variant: ident, $bytes_per_block: expr, $component_count: expr, $component_type: expr, $depth: expr, $stencil: expr, $compression_x: expr, $compression_y: expr),
    )*) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        #[allow(non_camel_case_types)]
        pub enum Format {
            $(
                $Variant,
            )*
        }

        impl Format {
            #[inline]
            pub fn bytes_per_block(&self) -> usize {
                match *self {
                    $(
                        Self::$Variant => $bytes_per_block,
                    )*
                }
            }

            #[inline]
            pub fn compute_block_count(&self, width: u32, height: u32) -> usize {
                let (x, y) = match *self {
                    $(
                        Self::$Variant => ($compression_x, $compression_y),
                    )*
                };

                ((width + (x - 1)) / x) as usize * ((height + (y - 1)) / y) as usize
            }

            #[inline]
            pub fn compute_byte_count(&self, width: u32, height: u32) -> usize {
                self.bytes_per_block() * self.compute_block_count(width, height)
            }
        }
    }
}

impl_format! {
    (BC1_UNORM_RGB,   8, 3, ComponentType::UNORM, false, false, 4, 4),
    (BC1_UNORM_RGBA,  8, 4, ComponentType::UNORM, false, false, 4, 4),
    (BC2_UNORM_RGBA, 16, 4, ComponentType::UNORM, false, false, 4, 4),
    (BC3_UNORM_RGBA, 16, 4, ComponentType::UNORM, false, false, 4, 4),
    (BC4_UNORM_R,     8, 1, ComponentType::UNORM, false, false, 4, 4),
    (BC4_SNORM_R,     8, 1, ComponentType::SNORM, false, false, 4, 4),
    (BC5_UNORM_RG,   16, 2, ComponentType::UNORM, false, false, 4, 4),
    (BC5_SNORM_RG,   16, 2, ComponentType::SNORM, false, false, 4, 4),
}

impl File {
    pub fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let header = FileHeader::from(RawFileHeader::parse(reader)?);

        struct LayerState {
            byte_offset: usize,
            width: u32,
            height: u32,
        }

        let mut state = LayerState {
            byte_offset: 0,
            width: header.width,
            height: header.height,
        };

        let layers: Vec<Layer> = (0..header.mipmap_count.max(1))
            .map({
                let state = &mut state;
                let pixel_format = &header.pixel_format;
                move |_layer_index| {
                    let layer = Layer {
                        byte_offset: state.byte_offset,
                        byte_count: pixel_format.compute_byte_count(state.width, state.height),
                        width: state.width,
                        height: state.height,
                    };
                    state.byte_offset += layer.byte_count;
                    state.width = std::cmp::max(1, state.width / 2);
                    state.height = std::cmp::max(1, state.height / 2);

                    layer
                }
            })
            .collect();

        assert_eq!(1, state.width);
        assert_eq!(1, state.height);

        let bytes = unsafe {
            let mut bytes = Vec::<u8>::with_capacity(state.byte_offset);
            reader.read_exact(std::slice::from_raw_parts_mut(bytes.as_mut_ptr(), state.byte_offset))?;
            bytes.set_len(state.byte_offset);
            bytes
        };

        Ok(Self { header, layers, bytes })
    }
}

#[macro_export]
macro_rules! dds_impl_gl_ext {
    () => {
        pub trait FormatExt {
            fn to_gl_internal_format(&self, srgb: bool) -> gl::InternalFormat;
        }

        $crate::dds_impl_gl_ext!(@format {
            (BC1_UNORM_RGB,  gl::COMPRESSED_RGB_S3TC_DXT1_EXT .into(), gl::COMPRESSED_SRGB_S3TC_DXT1_EXT.into()      ),
            (BC1_UNORM_RGBA, gl::COMPRESSED_RGBA_S3TC_DXT1_EXT.into(), gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT.into()),
            (BC2_UNORM_RGBA, gl::COMPRESSED_RGBA_S3TC_DXT3_EXT.into(), gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT.into()),
            (BC3_UNORM_RGBA, gl::COMPRESSED_RGBA_S3TC_DXT5_EXT.into(), gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT.into()),
            (BC4_UNORM_R,    gl::COMPRESSED_RED_RGTC1         .into(), panic!("Format doesn't support srgb")         ),
            (BC4_SNORM_R,    gl::COMPRESSED_SIGNED_RED_RGTC1  .into(), panic!("Format doesn't support srgb")         ),
            (BC5_UNORM_RG,   gl::COMPRESSED_RG_RGTC2          .into(), panic!("Format doesn't support srgb")         ),
            (BC5_SNORM_RG,   gl::COMPRESSED_SIGNED_RG_RGTC2   .into(), panic!("Format doesn't support srgb")         ),
        });
    };
    (@format {$(
         ($Variant: ident, $linear: expr, $gamma: expr),
    )*}) => {
        impl FormatExt for $crate::Format {
            #[inline]
            fn to_gl_internal_format(&self, srgb: bool) -> gl::InternalFormat {
                match *self {
                    $(
                        Self::$Variant => if srgb { $gamma } else { $linear },
                    )*
                }
            }
        }
    };
}
