use crate::{infallible::sizes::Bin, result::size_pairs::SizePair};

pub struct BinBin;

impl SizePair for BinBin {
    type S1 = Bin;

    type S2 = Bin;
}
