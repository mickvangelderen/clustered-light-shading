use super::color::*;
use belene::*;

#[repr(C, packed(1))]
pub struct Block {
    pub alphas: u64le,
    pub colors: [u16le; 2],
    pub color_indices: u32le,
}

impl Block {
    pub fn to_rgba_8888(&self) -> [[RGBA_8888; 4]; 4] {
        let mut pixels: [[RGBA_8888; 4]; 4] = Default::default();

        let alphas = self.alphas.to_ne();
        let color_indices = self.color_indices.to_ne();

        let color_table: [RGB_888; 4] = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors: [RGB_888; 2] = [RGB_565(colors_u16[0]).into(), RGB_565(colors_u16[1]).into()];

            [
                colors[0],
                colors[1],
                RGB_888::weigh([2, 1], colors),
                RGB_888::weigh([1, 2], colors),
            ]
        };

        for i in 0..16 {
            let ci = (color_indices >> (i * 2)) & 0b11;
            let RGB_888 { r, g, b } = color_table[ci as usize];
            let a = u5_to_u8(((alphas >> (i * 4)) & 0b1111) as u8);
            pixels[i / 4][i % 4] = RGBA_8888 { r, g, b, a };
        }

        pixels
    }
}
