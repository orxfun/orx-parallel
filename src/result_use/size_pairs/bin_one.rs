use crate::infallible_use::{XapBin, XapOne, XapUse};
use crate::result_use::size_pairs::size_pair_use_res::SizePairUseRes;
use crate::sizes::BinOne;

impl SizePairRes for BinOne {
    type XapResResult<M, E, X1, X2>
        = Option<Result<X2::O, E>>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.bin_value(i);
        a.map(|a| a.map(|a| x2.one_value(a)))
    }
}
