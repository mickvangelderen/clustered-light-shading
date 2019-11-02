#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub struct RGBA_5650(pub u16);

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub struct RGBA_8880 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub struct RGBA_8888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<RGBA_5650> for RGBA_8880 {
    fn from(c: RGBA_5650) -> Self {
        let n = c.0;
        Self {
            r: ((((n >> 11) & 0b011111) as u32 * 255) / 31) as u8,
            g: ((((n >> 05) & 0b111111) as u32 * 255) / 63) as u8,
            b: ((((n >> 00) & 0b011111) as u32 * 255) / 31) as u8,
        }
    }
}

impl From<RGBA_8880> for RGBA_5650 {
    fn from(c: RGBA_8880) -> Self {
        let r = (((c.r as u32 * 31 + 30) / 255) & 0b011111) as u16;
        let g = (((c.g as u32 * 63 + 62) / 255) & 0b111111) as u16;
        let b = (((c.b as u32 * 31 + 30) / 255) & 0b011111) as u16;
        Self((r << 11) | (g << 05) | (b << 00))
    }
}

impl RGBA_8880 {
    #[inline]
    pub fn to_bytes(&self) -> [u8; 3] {
        let Self { r, g, b } = *self;
        [r, g, b]
    }

    #[inline]
    pub fn mix(w0: u32, c0: Self, w1: u32, c1: Self) -> Self {
        let ws = w0 + w1;
        Self {
            r: ((c0.r as u32 * w0 + c1.r as u32 * w1) / ws) as u8,
            g: ((c0.g as u32 * w0 + c1.g as u32 * w1) / ws) as u8,
            b: ((c0.b as u32 * w0 + c1.b as u32 * w1) / ws) as u8,
        }
    }
}

impl RGBA_8888 {
    #[inline]
    pub fn to_bytes(&self) -> [u8; 4] {
        let Self { r, g, b, a } = *self;
        [r, g, b, a]
    }

    #[inline]
    pub fn mix(w0: u32, c0: Self, w1: u32, c1: Self) -> Self {
        let ws = w0 + w1;
        Self {
            r: ((c0.r as u32 * w0 + c1.r as u32 * w1) / ws) as u8,
            g: ((c0.g as u32 * w0 + c1.g as u32 * w1) / ws) as u8,
            b: ((c0.b as u32 * w0 + c1.b as u32 * w1) / ws) as u8,
            a: ((c0.a as u32 * w0 + c1.a as u32 * w1) / ws) as u8,
        }
    }
}

