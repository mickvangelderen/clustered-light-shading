use super::common::*;
use belene::*;

#[repr(C, packed(1))]
pub struct Block {
    pub alphas: [u8; 2],
    pub alpha_indices: [u8; 6],
    pub colors: [u16le; 2],
    pub color_indices: u32le,
}

fn weigh_u8(weights: [u32; 2], values: [u8; 2]) -> u8 {
    let [w0, w1] = weights;
    let [n0, n1] = values;
    (w0 * (n0 as u32) + w1 * (n1 as u32) / (w0 + w1)) as u8
}

impl Block {
    pub fn to_rgba_8888(&self) -> [[RGBA_8888; 4]; 4] {
        let mut pixels: [[RGBA_8888; 4]; 4] = Default::default();

        let alphas = self.alphas;

        // Put the bytes in a u64 to easily access bits across byte-boundaries,
        let alpha_indices = u64::from_le_bytes([
            self.alpha_indices[0],
            self.alpha_indices[1],
            self.alpha_indices[2],
            self.alpha_indices[3],
            self.alpha_indices[4],
            self.alpha_indices[5],
            0,
            0,
        ]);

        let alpha_table: [u8; 8] = {
            if alphas[0] > alphas[1] {
                [
                    alphas[0],
                    alphas[1],
                    weigh_u8([6, 1], alphas),
                    weigh_u8([5, 2], alphas),
                    weigh_u8([4, 3], alphas),
                    weigh_u8([3, 4], alphas),
                    weigh_u8([2, 5], alphas),
                    weigh_u8([1, 6], alphas),
                ]
            } else {
                [
                    alphas[0],
                    alphas[1],
                    weigh_u8([4, 1], alphas),
                    weigh_u8([3, 2], alphas),
                    weigh_u8([2, 3], alphas),
                    weigh_u8([1, 4], alphas),
                    0,
                    255,
                ]
            }
        };

        let color_indices = self.color_indices.to_ne();

        let color_table: [RGBA_8880; 4] = {
            let colors_u16 = [self.colors[0].to_ne(), self.colors[1].to_ne()];
            let colors: [RGBA_8880; 2] = [RGBA_5650(colors_u16[0]).into(), RGBA_5650(colors_u16[1]).into()];

            [
                colors[0],
                colors[1],
                RGBA_8880::mix(2, colors[0], 1, colors[1]),
                RGBA_8880::mix(1, colors[0], 2, colors[1]),
            ]
        };

        for i in 0..16 {
            let ai = (alpha_indices >> (i * 3)) & 0b111;
            let a = alpha_table[ai as usize];
            let rgbi = (color_indices >> (i * 2)) & 0b11;
            let RGBA_8880 { r, g, b } = color_table[rgbi as usize];
            pixels[i / 4][i % 4] = RGBA_8888 { r, g, b, a };
        }

        pixels
    }
}
