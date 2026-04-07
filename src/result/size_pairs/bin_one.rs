use crate::infallible::sizes::{Bin, One};
use crate::result::size_pairs::SizePair;

pub struct BinOne;

impl SizePair for BinOne {
    type S1 = Bin;

    type S2 = One;
}
