use crate::infallible::{Xap, XapBin};
use crate::sizes::{Bin, size_pair::SizePair, size_pairs::BinMany};

#[derive(Clone, Copy, Default)]
pub struct BinBin;

impl SizePair for BinBin {
    type S1 = Bin;

    type S2 = Bin;

    type ThenBin = BinBin;

    type ThenMany = BinMany;

    // option

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
