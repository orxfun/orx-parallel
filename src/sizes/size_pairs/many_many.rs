use crate::sizes::{Many, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct ManyMany;

impl SizePair for ManyMany {
    type S1 = Many;

    type S2 = Many;

    type ThenBin = ManyMany;

    type ThenMany = ManyMany;
}
