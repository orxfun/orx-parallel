use crate::sizes::{Bin, One, size_pair::SizePair, size_pairs::OneMany};

#[derive(Clone, Copy, Default)]
pub struct OneBin;

impl SizePair for OneBin {
    type S1 = One;

    type S2 = Bin;

    type ThenBin = OneBin;

    type ThenMany = OneMany;
}
