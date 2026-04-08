use crate::sizes::{Many, One, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct OneMany;

impl SizePair for OneMany {
    type S1 = One;

    type S2 = Many;

    type ThenBin = OneMany;

    type ThenMany = OneMany;
}
