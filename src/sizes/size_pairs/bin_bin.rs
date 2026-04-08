use crate::sizes::{Bin, size_pair::SizePair, size_pairs::BinMany};

#[derive(Clone, Copy, Default)]
pub struct BinBin;

impl SizePair for BinBin {
    type S1 = Bin;

    type S2 = Bin;

    type ThenBin = BinBin;

    type ThenMany = BinMany;
}
