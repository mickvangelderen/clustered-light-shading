use belene::*;
use gl_typed as gl;
use std::io;

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
    pub caps: [u32le; 4],
    pub _reserved_1: u32le,
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
    pub pixel_format: Format,
}

impl From<RawFileHeader> for FileHeader {
    fn from(header: RawFileHeader) -> Self {
        let mipmap_count = header.mipmap_count.to_ne();
        let width = header.width.to_ne();
        let height = header.height.to_ne();
        let depth = header.depth.to_ne();
        let pixel_format_flags = header.pixel_format.flags.to_ne();

        let pixel_format: Format = if pixel_format_flags & PixelFormatFlags::FOURCC == PixelFormatFlags::FOURCC {
            match header.pixel_format.four_cc {
                FOURCC_DXT1 => Format::BC1_UNORM_RGB,
                FOURCC_DXT2 | FOURCC_DXT3 => Format::BC2_UNORM_RGBA,
                FOURCC_DXT4 | FOURCC_DXT5 => Format::BC3_UNORM_RGBA,
                FOURCC_ATI1 | FOURCC_BC4U => Format::BC4_UNORM_R,
                FOURCC_BC4S => Format::BC4_SNORM_R,
                FOURCC_ATI2 | FOURCC_BC5U => Format::BC5_UNORM_RG,
                FOURCC_BC5S => Format::BC5_SNORM_RG,
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

pub const FOURCC_DXT1: [u8; 4] = *b"DXT1";
pub const FOURCC_DXT2: [u8; 4] = *b"DXT2";
pub const FOURCC_DXT3: [u8; 4] = *b"DXT3";
pub const FOURCC_DXT4: [u8; 4] = *b"DXT4";
pub const FOURCC_DXT5: [u8; 4] = *b"DXT5";
pub const FOURCC_ATI1: [u8; 4] = *b"ATI1";
pub const FOURCC_BC4U: [u8; 4] = *b"BC4U";
pub const FOURCC_BC4S: [u8; 4] = *b"BC4S";
pub const FOURCC_ATI2: [u8; 4] = *b"ATI2";
pub const FOURCC_BC5U: [u8; 4] = *b"BC5U";
pub const FOURCC_BC5S: [u8; 4] = *b"BC5S";
pub const FOURCC_RGBG: [u8; 4] = *b"RGBG";
pub const FOURCC_GRGB: [u8; 4] = *b"GRGB";
pub const FOURCC_YUY2: [u8; 4] = *b"YUY2";

// CompressedRed = COMPRESSED_RED,
// CompressedRg = COMPRESSED_RG,
// CompressedRgb = COMPRESSED_RGB,
// CompressedRgba = COMPRESSED_RGBA,
// CompressedSrgb = COMPRESSED_SRGB,
// CompressedSrgbAlpha = COMPRESSED_SRGB_ALPHA,
// CompressedRedRgtc1 = COMPRESSED_RED_RGTC1,
// CompressedSignedRedRgtc1 = COMPRESSED_SIGNED_RED_RGTC1,
// CompressedRgRgtc2 = COMPRESSED_RG_RGTC2,
// CompressedSignedRgRgtc2 = COMPRESSED_SIGNED_RG_RGTC2,
// CompressedRgbaBptcUnorm = COMPRESSED_RGBA_BPTC_UNORM,
// CompressedSrgbAlphaBptcUnorm = COMPRESSED_SRGB_ALPHA_BPTC_UNORM,
// CompressedRgbBptcSignedFloat = COMPRESSED_RGB_BPTC_SIGNED_FLOAT,
// CompressedRgbBptcUnsignedFloat = COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT,

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

// from falcor
// // Flags
// static const uint32_t kCapsMask = 0x1;
// static const uint32_t kHeightMask = 0x2;
// static const uint32_t kWidthMask = 0x4;
// static const uint32_t kPitchMask = 0x8;
// static const uint32_t kPixelFormatMask = 0x1000;
// static const uint32_t kMipCountMask = 0x20000;
// static const uint32_t kLinearSizeMask = 0x80000;
// static const uint32_t kDepthMask = 0x800000;

// // Caps[0]
// static const uint32_t kCapsComplexMask = 0x8;
// static const uint32_t kCapsMipMapMask = 0x400000;
// static const uint32_t kCapsTextureMask = 0x1000;

// // Caps[1]
// static const uint32_t kCaps2CubeMapMask = 0x200;
// static const uint32_t kCaps2CubeMapPosXMask = 0x400;
// static const uint32_t kCaps2CubeMapNegXMask = 0x800;
// static const uint32_t kCaps2CubeMapPosYMask = 0x1000;
// static const uint32_t kCaps2CubeMapNegYMask = 0x2000;
// static const uint32_t kCaps2CubeMapPosZMask = 0x4000;
// static const uint32_t kCaps2CubeMapNegZMask = 0x8000;
// static const uint32_t kCaps2VolumeMask = 0x200000;

macro_rules! impl_format {
    ($(
        ($Variant: ident, $bytes_per_block: expr, $component_count: expr, $component_type: expr, $depth: expr, $stencil: expr, $compression_x: expr, $compression_y: expr, $gl_internal_format: expr, $gl_format: expr, $gl_component_type: expr),
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

            // #[inline]
            // pub fn component_count(&self) -> u8 {
            //     match *self {
            //         $(
            //             Self::$Variant => $component_count,
            //         )*
            //     }
            // }

            // #[inline]
            // pub fn component_type(&self) -> ComponentType {
            //     match *self {
            //         $(
            //             Self::$Variant => $component_type,
            //         )*
            //     }
            // }

            #[inline]
            pub fn to_gl_internal_format(&self) -> gl::InternalFormat {
                match *self {
                    $(
                        Self::$Variant => $gl_internal_format.into(),
                    )*
                }
            }
        }
    }
}

pub enum ComponentType {
    FLOAT,
    UNORM,
    SNORM,
    UINT,
    SINT,
}

impl_format! {
    (BC1_UNORM_RGB,           8,           3,  ComponentType::UNORM,      false,  false,        4, 4, gl::COMPRESSED_RGB_S3TC_DXT1_EXT,  gl::RGB,  gl::NONE),
    (BC1_UNORM_RGBA,          8,           4,  ComponentType::UNORM,      false,  false,        4, 4, gl::COMPRESSED_RGBA_S3TC_DXT1_EXT, gl::RGBA, gl::NONE),
    (BC2_UNORM_RGBA,         16,           4,  ComponentType::UNORM,      false,  false,        4, 4, gl::COMPRESSED_RGBA_S3TC_DXT3_EXT, gl::RGBA, gl::NONE),
    (BC3_UNORM_RGBA,         16,           4,  ComponentType::UNORM,      false,  false,        4, 4, gl::COMPRESSED_RGBA_S3TC_DXT5_EXT, gl::RGBA, gl::NONE),
    (BC4_UNORM_R,             8,           1,  ComponentType::UNORM,      false,  false,        4, 4, gl::COMPRESSED_RED_RGTC1,          gl::RED,  gl::NONE),
    (BC4_SNORM_R,             8,           1,  ComponentType::SNORM,      false,  false,        4, 4, gl::COMPRESSED_SIGNED_RED_RGTC1,   gl::RED,  gl::NONE),
    (BC5_UNORM_RG,           16,           2,  ComponentType::UNORM,      false,  false,        4, 4, gl::COMPRESSED_RG_RGTC2,           gl::RG,   gl::NONE),
    (BC5_SNORM_RG,           16,           2,  ComponentType::SNORM,      false,  false,        4, 4, gl::COMPRESSED_SIGNED_RG_RGTC2,    gl::RG,   gl::NONE),
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

        let layers: Vec<Layer> = (0..header.mipmap_count)
            .into_iter()
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
            reader.read_exact(std::slice::from_raw_parts_mut(bytes.as_mut_ptr(), bytes.capacity()))?;
            bytes.set_len(bytes.capacity());
            bytes
        };

        Ok(Self { header, layers, bytes })
    }
}
