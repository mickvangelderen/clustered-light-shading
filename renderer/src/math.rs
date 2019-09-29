pub trait CeiledDiv {
    fn ceiled_div(self, rhs: Self) -> Self;
}

impl<T> CeiledDiv for T
where
    T: Copy
        + std::ops::Div<Output = Self>
        + std::ops::Add<Output = Self>
        + std::ops::Sub<Output = Self>
        + One,
{
    fn ceiled_div(self, rhs: Self) -> Self {
        (self + rhs - T::ONE) / rhs
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
