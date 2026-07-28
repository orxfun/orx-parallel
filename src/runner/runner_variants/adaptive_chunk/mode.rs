use core::sync::atomic::{AtomicUsize, Ordering};

const MODE_EXPLORE: usize = 0;
const MODE_FIXED: usize = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Explore,
    Fixed,
}

pub struct AtomicMode(AtomicUsize);

impl AtomicMode {
    pub fn new_explore() -> Self {
        Self(AtomicUsize::new(MODE_EXPLORE))
    }

    pub fn complete_exploration(&self) {
        self.0.store(MODE_FIXED, Ordering::Relaxed);
    }

    pub fn mode(&self) -> Mode {
        match self.0.load(Ordering::Relaxed) {
            MODE_EXPLORE => Mode::Explore,
            _ => Mode::Fixed,
        }
    }
}
