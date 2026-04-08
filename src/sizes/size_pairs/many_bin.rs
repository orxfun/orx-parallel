use crate::sizes::{Bin, Many, size_pair::SizePair, size_pairs::ManyMany};

#[derive(Clone, Copy, Default)]
pub struct ManyBin;

impl SizePair for ManyBin {
    type S1 = Many;

    type S2 = Bin;

    type ThenBin = ManyBin;

    type ThenMany = ManyMany;
}
