use crate::num::*;
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
                FOURCC_DXT1 => Format::BC1_UNORM,
                FOURCC_DXT2 | FOURCC_DXT3 => Format::BC2_UNORM,
                FOURCC_DXT4 | FOURCC_DXT5 => Format::BC3_UNORM,
                FOURCC_ATI1 | FOURCC_BC4U => Format::BC4_UNORM,
                FOURCC_BC4S => Format::BC4_SNORM,
                FOURCC_ATI2 | FOURCC_BC5U => Format::BC5_UNORM,
                FOURCC_BC5S => Format::BC5_SNORM,
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

fn compute_block_count(width: u32, height: u32) -> u32 {
    ((width + 3) / 4) * ((height + 3) / 4)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Format {
    // R32G32B32A32_TYPELESS,
    // R32G32B32A32_FLOAT,
    // R32G32B32A32_UINT,
    // R32G32B32A32_SINT,
    // R32G32B32_TYPELESS,
    // R32G32B32_FLOAT,
    // R32G32B32_UINT,
    // R32G32B32_SINT,
    // R16G16B16A16_TYPELESS,
    // R16G16B16A16_FLOAT,
    // R16G16B16A16_UNORM,
    // R16G16B16A16_UINT,
    // R16G16B16A16_SNORM,
    // R16G16B16A16_SINT,
    // R32G32_TYPELESS,
    // R32G32_FLOAT,
    // R32G32_UINT,
    // R32G32_SINT,
    // R32G8X24_TYPELESS,
    // D32_FLOAT_S8X24_UINT,
    // R32_FLOAT_X8X24_TYPELESS,
    // X32_TYPELESS_G8X24_UINT,
    // R10G10B10A2_TYPELESS,
    // R10G10B10A2_UNORM,
    // R10G10B10A2_UINT,
    // R11G11B10_FLOAT,
    // R8G8B8A8_TYPELESS,
    // R8G8B8A8_UNORM,
    // R8G8B8A8_UNORM_SRGB,
    // R8G8B8A8_UINT,
    // R8G8B8A8_SNORM,
    // R8G8B8A8_SINT,
    // R16G16_TYPELESS,
    // R16G16_FLOAT,
    // R16G16_UNORM,
    // R16G16_UINT,
    // R16G16_SNORM,
    // R16G16_SINT,
    // R32_TYPELESS,
    // D32_FLOAT,
    // R32_FLOAT,
    // R32_UINT,
    // R32_SINT,
    // R24G8_TYPELESS,
    // D24_UNORM_S8_UINT,
    // R24_UNORM_X8_TYPELESS,
    // X24_TYPELESS_G8_UINT,
    // R8G8_TYPELESS,
    // R8G8_UNORM,
    // R8G8_UINT,
    // R8G8_SNORM,
    // R8G8_SINT,
    // R16_TYPELESS,
    // R16_FLOAT,
    // D16_UNORM,
    // R16_UNORM,
    // R16_UINT,
    // R16_SNORM,
    // R16_SINT,
    // R8_TYPELESS,
    // R8_UNORM,
    // R8_UINT,
    // R8_SNORM,
    // R8_SINT,
    // A8_UNORM,
    // R1_UNORM,
    // R9G9B9E5_SHAREDEXP,
    // R8G8_B8G8_UNORM,
    // G8R8_G8B8_UNORM,
    // BC1_TYPELESS,
    BC1_UNORM,
    // BC1_UNORM_SRGB,
    // BC2_TYPELESS,
    BC2_UNORM,
    // BC2_UNORM_SRGB,
    // BC3_TYPELESS,
    BC3_UNORM,
    // BC3_UNORM_SRGB,
    // BC4_TYPELESS,
    BC4_UNORM,
    BC4_SNORM,
    // BC5_TYPELESS,
    BC5_UNORM,
    BC5_SNORM,
    // B5G6R5_UNORM,
    // B5G5R5A1_UNORM,
    // B8G8R8A8_UNORM,
    // B8G8R8X8_UNORM,
    // R10G10B10_XR_BIAS_A2_UNORM,
    // B8G8R8A8_TYPELESS,
    // B8G8R8A8_UNORM_SRGB,
    // B8G8R8X8_TYPELESS,
    // B8G8R8X8_UNORM_SRGB,
    // BC6H_TYPELESS,
    // BC6H_UF16,
    // BC6H_SF16,
    // BC7_TYPELESS,
    // BC7_UNORM,
    // BC7_UNORM_SRGB,
    // AYUV,
    // Y410,
    // Y416,
    // NV12,
    // P010,
    // P016,
    // F420_OPAQUE,
    // YUY2,
    // Y210,
    // Y216,
    // NV11,
    // AI44,
    // IA44,
    // P8,
    // A8P8,
    // B4G4R4A4_UNORM,
    // P208,
    // V208,
    // V408,
    // FORCE_UINT,
}

impl Format {
    fn block_byte_size(&self) -> usize {
        match *self {
            Self::BC1_UNORM => 8,
            Self::BC2_UNORM => 16,
            Self::BC3_UNORM => 16,
            Self::BC4_UNORM => 16,
            Self::BC4_SNORM => 16,
            Self::BC5_UNORM => 16,
            Self::BC5_SNORM => 16,
        }
    }
}

impl RawFile {
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
                move |_layer_index| {
                    let layer = Layer {
                        byte_offset: state.byte_offset,
                        byte_count: compute_block_count(state.width, state.height) as usize
                            * header.pixel_format.block_byte_size(),
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

    // case FOURCC_DXT1:
    // genericImage->format = GL_COMPRESSED_RGBA_S3TC_DXT1_EXT;
    // break;
    // case FOURCC_DXT3:
    // genericImage->format = GL_COMPRESSED_RGBA_S3TC_DXT3_EXT;
    // break;
    // case FOURCC_DXT5:
    // genericImage->format = GL_COMPRESSED_RGBA_S3TC_DXT5_EXT;
    // break;
    // default:
    // free(genericImage->pixels);
    // free(genericImage);
    // ret
}
