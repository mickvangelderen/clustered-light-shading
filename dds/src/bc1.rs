use super::color::*;
use belene::*;

#[repr(C, packed)]
pub struct Block {
    pub colors: [u16le; 2],
    pub color_indices: u32le,
}

impl Block {
    pub fn to_rgba_8880(&self) -> [[RGB_888; 4]; 4] {
        let mut pixels: [[RGB_888; 4]; 4] = Default::default();

        let color_indices = self.color_indices.to_ne();

        let color_table: [RGB_888; 4] = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors: [RGB_888; 2] = [RGB_565(colors_u16[0]).into(), RGB_565(colors_u16[1]).into()];

            if colors_u16[0] > colors_u16[1] {
                [
                    colors[0],
                    colors[1],
                    RGB_888::weigh([2, 1], colors),
                    RGB_888::weigh([1, 2], colors),
                ]
            } else {
                [
                    colors[0],
                    colors[1],
                    RGB_888::weigh([1, 1], colors),
                    RGB_888 { r: 0, g: 0, b: 0 },
                ]
            }
        };

        for i in 0..16 {
            let ci = (color_indices >> (i * 2)) & 0b11;
            pixels[i / 4][i % 4] = color_table[ci as usize];
        }

        pixels
    }
}
