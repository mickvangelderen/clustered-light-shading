const LENGTH: usize = 124;

pub struct MovingAverageF32 {
    index: u32,
    _pad0: [u32; 3],
    samples: [f32; LENGTH],
}

impl MovingAverageF32 {
    pub fn new(default: f32) -> Self {
        MovingAverageF32 {
            index: 0,
            _pad0: [0; 3],
            samples: [default; LENGTH],
        }
    }

    pub fn submit(&mut self, sample: f32) {
        unsafe {
            *self.samples.get_unchecked_mut(self.index as usize) = sample;
            self.index += 1;
            if self.index == self.samples.len() as u32 {
                self.index = 0;
            }
        }
    }

    pub fn compute(&self) -> f32 {
        let sum: f32 = self.samples.iter().sum();
        let count = self.samples.len() as f32;
        sum / count
    }
}
