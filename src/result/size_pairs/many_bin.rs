use crate::infallible::sizes::{Bin, Many};
use crate::result::size_pairs::SizePair;

pub struct ManyBin;

impl SizePair for ManyBin {
    type S1 = Many;

    type S2 = Bin;
}
