use super::common::*;
use belene::*;

#[repr(C, packed(1))]
pub struct Block {
    pub alpha: u64le,
    pub colors: [u16le; 2],
    pub indices: u32le,
}

impl Block {
    pub fn to_rgba_8888(&self) -> [[RGBA_8888; 4]; 4] {
        let mut pixels: [[RGBA_8888; 4]; 4] = Default::default();

        let alpha = self.alpha.to_ne();
        let indices = self.indices.to_ne();

        let table: [RGBA_8880; 4] = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors: [RGBA_8880; 2] = [RGBA_5650(colors_u16[0]).into(), RGBA_5650(colors_u16[1]).into()];

            [
                colors[0],
                colors[1],
                RGBA_8880::mix(2, colors[0], 1, colors[1]),
                RGBA_8880::mix(1, colors[0], 2, colors[1]),
            ]
        };

        for y in 0..4 {
            for x in 0..4 {
                let pi = y * 4 + x;
                let RGBA_8880 { r, g, b } = table[((indices >> (pi * 2)) & 0b11) as usize];
                let a = ((((alpha >> (pi * 4)) & 0b1111) as u32 * 255) / 31) as u8;
                pixels[y][x] = RGBA_8888 { r, g, b, a };
            }
        }

        pixels
    }
}
