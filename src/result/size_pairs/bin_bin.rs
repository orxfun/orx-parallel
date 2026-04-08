use crate::infallible::XapBin;
use crate::infallible::{Xap, sizes::Bin};
use crate::result::size_pairs::{BinMany, SizePair};

#[derive(Clone, Copy, Default)]
pub struct BinBin;

impl SizePair for BinBin {
    type S1 = Bin;

    type S2 = Bin;

    type ThenBin = BinBin;

    type ThenMany = BinMany;

    type Results<M, E, X1, X2>
        = Option<Result<X2::O, E>>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::Results<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        x1.bin_value(i).and_then(|a| match a {
            Ok(a) => x2.bin_value(a).map(Ok),
            Err(e) => Some(Err(e)),
        })
    }
}
