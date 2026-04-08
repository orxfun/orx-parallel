use crate::sizes::{Bin, Many, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct BinMany;

impl SizePair for BinMany {
    type S1 = Bin;

    type S2 = Many;

    type ThenBin = BinMany;

    type ThenMany = BinMany;
}
