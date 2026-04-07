use crate::infallible::Xap;
use crate::infallible::sizes::{Bin, One};
use crate::result::size_pairs::SizePair;

pub struct OneBin;

impl SizePair for OneBin {
    type S1 = One;

    type S2 = Bin;

    type Results<M, E, X1, X2>
        = Option<Result<X2::O, E>>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;
}
