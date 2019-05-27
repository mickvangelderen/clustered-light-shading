pub trait Clamp: Sized {
    fn clamp_range(self, range: (Self, Self)) -> Self;
}

impl<T> Clamp for T
where
    T: PartialOrd,
    T: Copy,
{
    fn clamp_range(self, range: (Self, Self)) -> Self {
        let (min, max) = range;
        if self > max {
            max
        } else if self < min {
            min
        } else {
            self
        }
    }
}
