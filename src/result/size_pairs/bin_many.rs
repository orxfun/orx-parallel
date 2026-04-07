use crate::infallible::sizes::{Bin, Many};
use crate::result::size_pairs::SizePair;

pub struct BinMany;

impl SizePair for BinMany {
    type S1 = Bin;

    type S2 = Many;
}
