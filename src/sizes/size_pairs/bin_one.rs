use crate::sizes::size_pair::SizePair;
use crate::sizes::size_pairs::{BinBin, BinMany};
use crate::sizes::{Bin, One};

#[derive(Clone, Copy, Default)]
pub struct BinOne;

impl SizePair for BinOne {
    type S1 = Bin;

    type S2 = One;

    type ThenBin = BinBin;

    type ThenMany = BinMany;
}
