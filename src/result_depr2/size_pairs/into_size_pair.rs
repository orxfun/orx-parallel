use crate::infallible::sizes::{Bin, Many, One, Size};
use crate::result_depr2::size_pairs::{BinOne, ManyOne, OneOne, SizePair};

pub trait IntoSizePair: Size {
    type ThenOne: SizePair<S1 = Self, S2 = One>;
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
