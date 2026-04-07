use crate::infallible::Xap;
use crate::infallible::sizes::{Bin, One};
use crate::result::size_pairs::SizePair;

pub struct BinOne;

impl SizePair for BinOne {
    type S1 = Bin;

    type S2 = One;

    type Results<M, E, X1, X2>
        = Option<Result<X2::O, E>>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;
}
