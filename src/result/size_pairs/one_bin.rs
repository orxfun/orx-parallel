use crate::infallible::sizes::{Bin, One};
use crate::result::size_pairs::SizePair;

pub struct OneBin;

impl SizePair for OneBin {
    type S1 = One;

    type S2 = Bin;
}
