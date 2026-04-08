use crate::infallible::Xap;
use crate::infallible::XapBin;
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::BinBin;

impl SizePairOpt for BinBin {
    type XapOptResult<M, X1, X2>
        = Option<Option<X2::O>>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline]
    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        x1.bin_value(i).and_then(|a| match a {
            Some(a) => x2.bin_value(a).map(Some),
            None => Some(None),
        })
    }
}
