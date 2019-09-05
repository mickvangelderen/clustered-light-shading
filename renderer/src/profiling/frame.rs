#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Frame(pub u64);

impl Frame {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl std::ops::Add<Self> for Frame {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Frame(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Self> for Frame {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Frame(self.0 - rhs.0)
    }
}

impl std::ops::Add<u64> for Frame {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Frame(self.0 + rhs)
    }
}

impl std::ops::Sub<u64> for Frame {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Frame(self.0 - rhs)
    }
}
