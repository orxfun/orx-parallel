use crate::infallible::Xap;
use crate::infallible::sizes::One;
use crate::result::size_pairs::SizePair;

pub struct OneOne;

impl SizePair for OneOne {
    type S1 = One;

    type S2 = One;

    type Results<M, E, X1, X2>
        = [Result<X2::O, E>; 1]
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;
}
