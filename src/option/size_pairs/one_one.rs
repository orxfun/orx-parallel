use crate::infallible::{Xap, XapOne};
use crate::result::size_pairs::SizePairRes;
use crate::sizes::OneOne;

impl SizePairRes for OneOne {
    type XapResResult<M, E, X1, X2>
        = [Result<X2::O, E>; 1]
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.one_value(i);
        [a.map(|a| x2.one_value(a))]
    }
}
