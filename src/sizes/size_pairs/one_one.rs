use crate::infallible::{Xap, XapOne};
use crate::sizes::size_pairs::{OneBin, OneMany};
use crate::sizes::{One, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct OneOne;

impl SizePair for OneOne {
    type S1 = One;

    type S2 = One;

    type ThenBin = OneBin;

    type ThenMany = OneMany;

    // option

    type XapOptResult<M, X1, X2>
        = [Option<X2::O>; 1]
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.one_value(i);
        [a.map(|a| x2.one_value(a))]
    }
}
