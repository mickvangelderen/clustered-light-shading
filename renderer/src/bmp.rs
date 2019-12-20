// #[repr(C)]
// struct FileHeader {
//     B: u8, // b'B'
//     M: u8, // b'M'
//     size: u32,
//     _pad: [u8; 4],
//     pixel_offset: u32,
// }

// #[repr(C)]
// struct BITMAPV5HEADER {
//     size: u32, // 124
//     width: u32,
//     height: u32,
//     planes: u16, // 1
//     bbp: u16,
//     compression: Compression,
//     image_size: u32,
//     ppm_x: u32, // 72dpi * 39.3701 = 2835
//     ppm_y: u32, // 72dpi * 39.3701 = 2835
//     palette_size: u32, // 0.
//     important_colors: u32 // 0.
// }

// #[repr(u32)]
// enum Compression {
//     BITFIELDS = 0x03,
// }
use std::io::Write;
use std::convert::TryFrom;

#[allow(unused)]
pub fn rgba_header(width: u32, height: u32) -> [u8; 122] {
    #[rustfmt::skip]
    let mut header: [u8; 122] = [
        0x42, 0x4d,             // Signature 'BM'
        0xCC, 0xCC, 0xCC, 0xCC, // Size
        0x00, 0x00,             // Unused
        0x00, 0x00,             // Unused
        0x7a, 0x00, 0x00, 0x00, // Offset to image data

        // 14
        0x6c, 0x00, 0x00, 0x00, // DIB header size
        0xCC, 0xCC, 0xCC, 0xCC, // Width
        0xCC, 0xCC, 0xCC, 0xCC, // Height
        0x01, 0x00,             // Planes (1)
        0x20, 0x00,             // Bits per pixel (32)
        0x03, 0x00, 0x00, 0x00, // Format (bitfield = use bitfields | no compression)
        0xCC, 0xCC, 0xCC, 0xCC, // Raw bitmap data size.
        0x13, 0x0B, 0x00, 0x00, // Horizontal print resolution (2835 = 72dpi * 39.3701)
        0x13, 0x0B, 0x00, 0x00, // Vertical print resolution (2835 = 72dpi * 39.3701)
        0x00, 0x00, 0x00, 0x00, // Colors in palette (none)
        0x00, 0x00, 0x00, 0x00, // Important colors (0 = all)
        0xFF, 0x00, 0x00, 0x00, // R bitmask
        0x00, 0xFF, 0x00, 0x00, // G bitmask
        0x00, 0x00, 0xFF, 0x00, // B bitmask
        0x00, 0x00, 0x00, 0xFF, // A bitmask
        0x42, 0x47, 0x52, 0x73, // sRGB color space
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [1]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [2]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [3]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [4]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [5]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [6]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [7]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [8]
        0x00, 0x00, 0x00, 0x00, // 4 bytes color space endpoints. [9]
        0x00, 0x00, 0x00, 0x00, // Unused Gamma X entry for color space
        0x00, 0x00, 0x00, 0x00, // Unused Gamma Y entry for color space
        0x00, 0x00, 0x00, 0x00, // Unused Gamma Z entry for color space
    ];

    let image_size = width * height * 4;
    let size = u32::try_from(std::mem::size_of_val(&header)).unwrap() + image_size;

    (&mut header[2..6]).write_all(&size.to_le_bytes()).unwrap();
    (&mut header[18..22]).write_all(&width.to_le_bytes()).unwrap();
    (&mut header[22..26]).write_all(&height.to_le_bytes()).unwrap();
    (&mut header[34..38]).write_all(&height.to_le_bytes()).unwrap();

    header
}
