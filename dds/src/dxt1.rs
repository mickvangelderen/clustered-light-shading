use super::common::*;
use belene::*;

#[repr(C, packed)]
pub struct Block {
    pub colors: [u16le; 2],
    pub table: u32le,
}

impl Block {
    pub fn to_rgba_8880(&self) -> [[RGBA_8880; 4]; 4] {
        let mut pixels: [[RGBA_8880; 4]; 4] = Default::default();

        let indices = self.table.to_ne();

        let table: [RGBA_8880; 4] = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors: [RGBA_8880; 2] = [RGBA_5650(colors_u16[0]).into(), RGBA_5650(colors_u16[1]).into()];

            if colors_u16[0] > colors_u16[1] {
                [
                    colors[0],
                    colors[1],
                    RGBA_8880::mix(2, colors[0], 1, colors[1]),
                    RGBA_8880::mix(1, colors[0], 2, colors[1]),
                ]
            } else {
                [
                    colors[0],
                    colors[1],
                    RGBA_8880::mix(1, colors[0], 1, colors[1]),
                    RGBA_8880 { r: 0, g: 0, b: 0 },
                ]
            }
        };

        for y in 0..4 {
            for x in 0..4 {
                let pi = y * 4 + x;
                pixels[y][x] = table[((indices >> (pi * 2)) & 0b11) as usize];
            }
        }

        pixels
    }
}
