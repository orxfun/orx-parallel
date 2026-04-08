use crate::infallible::{Xap, XapBin, XapOne};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::OneBin;

impl SizePairOpt for OneBin {
    type XapOptResult<M, X1, X2>
        = Option<Option<X2::O>>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        match x1.one_value(i) {
            Some(a) => x2.bin_value(a).map(Some),
            None => Some(None),
        }
    }
}
