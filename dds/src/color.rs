//! let M_a = 2^a - 1
//! Mapping [0, M_a] to [0.0, 1.0] we get f_a(n_a) = n_a / M_a
//! n_a / M_a = n_b / M_b
//! n_b = n_a * M_b / M_a
//!
//! Now to round the computation of n_b rather than floor.
//! n_b = floor(n_a * M_b / M_a + 1/2)
//! n_b = floor((n_a * 2 * M_b + M_a) / (2 * M_a))

/// Mersenne prime: 2^bits - 1.
macro_rules! M {
    ($bits: expr) => {
        ((1 << $bits) - 1)
    };
}

#[inline]
pub fn u5_to_u8(n5: u8) -> u8 {
    ((n5 as u16 * 2 * M!(8) + M!(5)) / (2 * M!(5))) as u8
}

#[inline]
pub fn u8_to_u5(n8: u8) -> u8 {
    ((n8 as u16 * 2 * M!(5) + M!(8)) / (2 * M!(8))) as u8
}

#[inline]
pub fn u6_to_u8(n6: u8) -> u8 {
    ((n6 as u16 * 2 * M!(8) + M!(6)) / (2 * M!(6))) as u8
}

#[inline]
pub fn u8_to_u6(n8: u8) -> u8 {
    ((n8 as u16 * 2 * M!(6) + M!(8)) / (2 * M!(8))) as u8
}

#[inline]
pub fn weigh_u8(weights: [u8; 2], values: [u8; 2]) -> u8 {
    let [w0, w1] = [weights[0] as u32, weights[1] as u32];
    let [n0, n1] = [values[0] as u32, values[1] as u32];
    ((w0 * n0 + w1 * n1) / (w0 + w1)) as u8
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct RGB_565(pub u16);

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct RGB_888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct RGBA_8888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<RGB_565> for RGB_888 {
    fn from(c: RGB_565) -> Self {
        let n = c.0;
        Self {
            r: u5_to_u8(((n >> 11) & 0b011111) as u8),
            g: u6_to_u8(((n >> 05) & 0b111111) as u8),
            b: u5_to_u8(((n >> 00) & 0b011111) as u8),
        }
    }
}

impl From<RGB_888> for RGB_565 {
    fn from(c: RGB_888) -> Self {
        let r = u8_to_u5(c.r) as u16;
        let g = u8_to_u6(c.g) as u16;
        let b = u8_to_u5(c.b) as u16;
        Self((r << 11) | (g << 05) | (b << 00))
    }
}

impl RGB_888 {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        let [r, g, b] = bytes;
        Self { r, g, b }
    }

    #[inline]
    pub fn to_bytes(&self) -> [u8; 3] {
        let Self { r, g, b } = *self;
        [r, g, b]
    }

    #[inline]
    pub fn weigh(weights: [u8; 2], values: [Self; 2]) -> Self {
        Self {
            r: weigh_u8(weights, [values[0].r, values[1].r]),
            g: weigh_u8(weights, [values[0].g, values[1].g]),
            b: weigh_u8(weights, [values[0].b, values[1].b]),
        }
    }
}

impl RGBA_8888 {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; 4]) {
        let [r, g, b, a] = bytes;
        Self { r, g, b, a };
    }

    #[inline]
    pub fn to_bytes(&self) -> [u8; 4] {
        let Self { r, g, b, a } = *self;
        [r, g, b, a]
    }

    #[inline]
    pub fn weigh(weights: [u8; 2], values: [Self; 2]) -> Self {
        Self {
            r: weigh_u8(weights, [values[0].r, values[1].r]),
            g: weigh_u8(weights, [values[0].g, values[1].g]),
            b: weigh_u8(weights, [values[0].b, values[1].b]),
            a: weigh_u8(weights, [values[0].a, values[1].a]),
        }
    }
}
