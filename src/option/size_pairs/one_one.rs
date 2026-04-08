use crate::infallible::{Xap, XapOne};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::OneOne;

impl SizePairOpt for OneOne {
    type XapOptResult<M, X1, X2>
        = [Option<X2::O>; 1]
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.one_value(i);
        [a.map(|a| x2.one_value(a))]
    }
}
