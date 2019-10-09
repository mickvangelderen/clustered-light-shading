#![allow(non_camel_case_types)]

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct u32le([u8; 4]);

impl u32le {
    #[inline]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    #[inline]
    pub fn to_ne(self) -> u32 {
        u32::from_le_bytes(self.0)
    }
}

use std::fmt;

impl fmt::Debug for u32le {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_ne().fmt(f)
    }
}
