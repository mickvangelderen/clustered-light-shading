macro_rules! impl_index {
    (pub $Name: ident($ty: ty), $from: ident, $to: ident) => {
        #[derive(serde::Deserialize, serde::Serialize, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $Name(pub(super) $ty);

        impl $Name {
            #[inline]
            pub fn $to(&self) -> $ty {
                self.0
            }

            #[inline]
            pub fn $from(index: $ty) -> Self {
                Self(index)
            }

            #[inline]
            pub fn increment(&mut self) {
                self.0 += 1;
            }
        }
    }
}

impl_index!(pub RunIndex(usize), from_usize, to_usize);
impl_index!(pub FrameIndex(usize), from_usize, to_usize);
impl_index!(pub SampleIndex(usize), from_usize, to_usize);
impl_index!(pub ProfilerIndex(usize), from_usize, to_usize);
