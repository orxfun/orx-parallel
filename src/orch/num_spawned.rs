/// Number of spawned threads to execute a parallel computation.
#[derive(Clone, Copy)]
pub struct NumSpawned(usize);

impl NumSpawned {
    /// Zero.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Adds one to the spawned thread count.
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Converts into usize.
    pub fn into_inner(self) -> usize {
        self.0
    }
}

impl core::ops::Rem for NumSpawned {
    type Output = usize;

    fn rem(self, rhs: Self) -> Self::Output {
        self.0 % rhs.0
    }
}
