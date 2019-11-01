use super::common::*;
use belene::*;

#[repr(C, packed)]
pub struct Block {
    pub colors: [u16le; 2],
    pub table: u32le,
}

impl Block {
    pub fn to_rgb_f32(&self) -> [[[f32; 3]; 4]; 4] {
        let mut pixels = [[[0f32; 3]; 4]; 4];

        let indices = self.table.to_ne();

        let table = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors_rgb_f32 = [R5G6B5(colors_u16[0]).to_rgb_f32(), R5G6B5(colors_u16[1]).to_rgb_f32()];

            if colors_u16[0] > colors_u16[1] {
                [
                    colors_rgb_f32[0],
                    colors_rgb_f32[1],
                    mix_rgb_f32(2.0, colors_rgb_f32[0], 1.0, colors_rgb_f32[1]),
                    mix_rgb_f32(1.0, colors_rgb_f32[0], 2.0, colors_rgb_f32[1]),
                ]
            } else {
                [
                    colors_rgb_f32[0],
                    colors_rgb_f32[1],
                    mix_rgb_f32(1.0, colors_rgb_f32[0], 1.0, colors_rgb_f32[1]),
                    [0.0; 3],
                ]
            }
        };

        for y in 0..4 {
            for x in 0..4 {
                let index = (indices >> (y * 8 + x * 2)) & 0b11;
                pixels[y][x] = table[index as usize];
            }
        }

        pixels
    }
}
