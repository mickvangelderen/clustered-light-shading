use std::fmt;

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct u32le(u32);

impl u32le {
    #[inline]
    pub const fn from_ne(val: u32) -> Self {
        Self(val.to_le())
    }

    #[inline]
    pub const fn to_ne(self) -> u32 {
        u32::from_le(self.0)
    }
}

impl fmt::Debug for u32le {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_ne().fmt(f)
    }
}
