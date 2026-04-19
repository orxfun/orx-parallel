use crate::sizes::{BinOne, Size};

#[derive(Clone, Copy, Default)]
pub struct Bin;

impl Size for Bin {
    type ThenBin = Bin;

    type IntoPair = BinOne;

    fn elem_len() -> Option<usize> {
        None
    }
}
