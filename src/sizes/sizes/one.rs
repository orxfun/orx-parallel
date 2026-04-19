use crate::sizes::{OneOne, Size, sizes::Bin};

#[derive(Clone, Copy, Default)]
pub struct One;

impl Size for One {
    type ThenBin = Bin;

    type ThenOne = OneOne;

    fn elem_len() -> Option<usize> {
        Some(1)
    }
}
