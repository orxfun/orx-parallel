use crate::sizes::{BinOne, Size};

#[derive(Clone, Copy, Default)]
pub struct Bin;

impl Size for Bin {
    type ThenBin = Bin;

    type ThenOne = BinOne;

    fn elem_len() -> Option<usize> {
        None
    }
}
