use crate::sizes::size_pairs::{OneBin, OneMany};
use crate::sizes::{One, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct OneOne;

impl SizePair for OneOne {
    type S1 = One;

    type S2 = One;

    type ThenBin = OneBin;

    type ThenMany = OneMany;
}
