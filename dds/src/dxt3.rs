use super::common::*;
use belene::*;

#[repr(C, packed)]
pub struct Block {
    pub alpha: u64le,
    pub colors: [u16le; 2],
    pub table: u32le,
}

impl Block {
    pub fn to_rgba_f32(&self) -> [[[f32; 4]; 4]; 4] {
        let mut pixels = [[[0f32; 4]; 4]; 4];

        let alpha = self.alpha.to_ne();
        let indices = self.table.to_ne();

        let table = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors_rgb_f32 = [R5G6B5(colors_u16[0]).to_rgb_f32(), R5G6B5(colors_u16[1]).to_rgb_f32()];

            [
                colors_rgb_f32[0],
                colors_rgb_f32[1],
                mix_rgb_f32(2.0, colors_rgb_f32[0], 1.0, colors_rgb_f32[1]),
                mix_rgb_f32(1.0, colors_rgb_f32[0], 2.0, colors_rgb_f32[1]),
            ]
        };

        for y in 0..4 {
            for x in 0..4 {
                let pi = y * 4 + x;
                let [r, g, b] = table[((indices >> (pi * 2)) & 0b11) as usize];
                let a = ((alpha >> (pi * 4)) & 0b1111) as f32 / 15.0;
                pixels[y][x] = [r, g, b, a];
            }
        }

        pixels
    }
}
