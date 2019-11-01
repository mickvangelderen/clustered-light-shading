pub struct R5G6B5(pub u16);

impl R5G6B5 {
    #[inline]
    pub fn to_rgb_f32(&self) -> [f32; 3] {
        let n = self.0;
        [
            ((n >> 11) & 0b011111) as f32 / 32.0,
            ((n >> 5) & 0b111111) as f32 / 64.0,
            ((n >> 0) & 0b011111) as f32 / 32.0,
        ]
    }

    #[inline]
    pub fn from_rgb_f32(rgb: [f32; 3]) -> Self {
        let r = ((rgb[0] * 31.0 + 0.5) as u16) & 0b011111;
        let g = ((rgb[1] * 63.0 + 0.5) as u16) & 0b111111;
        let b = ((rgb[2] * 31.0 + 0.5) as u16) & 0b011111;
        Self((r << 11) | (g << 5) | (b << 0))
    }
}

#[inline]
pub fn mix_rgb_f32(w0: f32, c0: [f32; 3], w1: f32, c1: [f32; 3]) -> [f32; 3] {
    let ws = w0 + w1;
    let w0n = w0 / ws;
    let w1n = w1 / ws;
    [
        w0n * c0[0] + w1n * c1[0],
        w0n * c0[1] + w1n * c1[1],
        w0n * c0[2] + w1n * c1[2],
    ]
}
