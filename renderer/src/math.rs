pub trait DivCeil {
    fn div_ceil(self, rhs: Self) -> Self;
}

impl<T> DivCeil for T
where
    T: Copy
        + std::ops::Div<Output = Self>
        + std::ops::Add<Output = Self>
        + std::ops::Rem<Output = Self>
        + Eq
        + Zero
        + One,
{
    fn div_ceil(self, rhs: Self) -> Self {
        self / rhs + if self % rhs == T::ZERO { T::ZERO } else { T::ONE }
    }
}

pub trait Zero {
    const ZERO: Self;
}

pub trait One {
    const ONE: Self;
}

macro_rules! impl_zero_one {
    ($($T: ty,)*) => {
        $(
            impl Zero for $T {
                const ZERO: Self = 0;
            }

            impl One for $T {
                const ONE: Self = 1;
            }
        )*
    }
}

impl_zero_one! {
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
}
