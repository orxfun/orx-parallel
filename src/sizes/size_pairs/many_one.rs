use crate::sizes::size_pairs::{ManyBin, ManyMany};
use crate::sizes::{Many, One, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct ManyOne;

impl SizePair for ManyOne {
    type S1 = Many;

    type S2 = One;

    type ThenBin = ManyBin;

    type ThenMany = ManyMany;
}
