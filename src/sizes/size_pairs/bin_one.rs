use crate::infallible::{Xap, XapBin, XapOne};
use crate::sizes::size_pair::SizePair;
use crate::sizes::size_pairs::{BinBin, BinMany};
use crate::sizes::{Bin, One};

#[derive(Clone, Copy, Default)]
pub struct BinOne;

impl SizePair for BinOne {
    type S1 = Bin;

    type S2 = One;

    type ThenBin = BinBin;

    type ThenMany = BinMany;

    // option

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
        let a = x1.bin_value(i);
        a.map(|a| a.map(|a| x2.one_value(a)))
    }
}
