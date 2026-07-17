/// Lock-free ring buffer for mono f32 audio samples.
pub struct AudioBuffer {
    data: [f32; Self::CAPACITY],
    write: usize,
    read: usize,
    count: usize,
}

impl AudioBuffer {
    const CAPACITY: usize = 4096;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [0.0f32; Self::CAPACITY],
            write: 0,
            read: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, samples: &[f32]) -> usize {
        let mut pushed = 0;
        for &s in samples {
            if self.count == Self::CAPACITY {
                break;
            }
            self.data[self.write] = s;
            self.write = (self.write + 1) % Self::CAPACITY;
            self.count += 1;
            pushed += 1;
        }
        pushed
    }

    #[must_use]
    pub fn pop(&mut self) -> Option<f32> {
        if self.count == 0 {
            return None;
        }
        let s = self.data[self.read];
        self.read = (self.read + 1) % Self::CAPACITY;
        self.count -= 1;
        Some(s)
    }

    #[must_use]
    pub fn available(&self) -> usize {
        self.count
    }

    pub fn fill_silence(&mut self, n: usize) {
        for _ in 0..n.min(Self::CAPACITY - self.count) {
            self.data[self.write] = 0.0;
            self.write = (self.write + 1) % Self::CAPACITY;
            self.count += 1;
        }
    }
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}
