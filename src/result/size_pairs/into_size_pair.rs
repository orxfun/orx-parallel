use crate::result::size_pairs::{BinOne, ManyOne, OneOne, SizePairRes};
use crate::sizes::{Bin, Many, One, Size};

pub trait IntoSizePair: Size {
    type ThenOne: SizePairRes<S1 = Self, S2 = One>;
}

impl IntoSizePair for One {
    type ThenOne = OneOne;
}

impl IntoSizePair for Bin {
    type ThenOne = BinOne;
}

impl IntoSizePair for Many {
    type ThenOne = ManyOne;
}
