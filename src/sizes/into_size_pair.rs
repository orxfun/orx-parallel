use crate::sizes::{Bin, BinOne, Many, ManyOne, One, OneOne, Size, SizePair};

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
