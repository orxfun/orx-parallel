use crate::sizes::{Size, sizes::Bin};

#[derive(Clone, Copy, Default)]
pub struct One;

impl Size for One {
    type ThenBin = Bin;

    fn size() -> Option<usize> {
        Some(1)
    }
}
