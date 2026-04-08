use crate::infallible::sizes::One;
use crate::infallible::{Xap, XapOne};
use crate::result::size_pairs::{OneBin, OneMany, SizePair};

#[derive(Clone, Copy, Default)]
pub struct OneOne;

impl SizePair for OneOne {
    type S1 = One;

    type S2 = One;

    type ThenBin = OneBin;

    type ThenMany = OneMany;

    type Results<M, E, X1, X2>
        = [Result<X2::O, E>; 1]
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::Results<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.one_value(i);
        [a.map(|a| x2.one_value(a))]
    }
}
