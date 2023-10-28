#![allow(non_camel_case_types)]

use std::fmt;

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct u32le([u8; 4]);

impl u32le {
    pub const fn from_ne(value: u32) -> Self {
        Self(value.to_le_bytes())
    }

    pub const fn to_ne(self) -> u32 {
        u32::from_le_bytes(self.0)
    }
}

impl From<u32> for u32le {
    fn from(value: u32) -> Self {
        Self::from_ne(value)
    }
}

impl From<u32le> for u32 {
    fn from(value: u32le) -> Self {
        value.to_ne()
    }
}

impl fmt::Debug for u32le {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        u32::from(*self).fmt(f)
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct u64le([u8; 8]);

impl u64le {
    pub const fn from_ne(value: u64) -> Self {
        Self(value.to_le_bytes())
    }

    pub const fn to_ne(self) -> u64 {
        u64::from_le_bytes(self.0)
    }
}

impl From<u64> for u64le {
    fn from(value: u64) -> Self {
        Self::from_ne(value)
    }
}

impl From<u64le> for u64 {
    fn from(value: u64le) -> Self {
        value.to_ne()
    }
}

impl fmt::Debug for u64le {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        u64::from(*self).fmt(f)
    }
}
