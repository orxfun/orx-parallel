use crate::infallible::{Xap, XapBin, XapOne};
use crate::result::size_pairs::{OneMany, SizePair};
use crate::sizes::{Bin, One};

#[derive(Clone, Copy, Default)]
pub struct OneBin;

impl SizePair for OneBin {
    type S1 = One;

    type S2 = Bin;

    type ThenBin = OneBin;

    type ThenMany = OneMany;

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
        match x1.one_value(i) {
            Ok(a) => x2.bin_value(a).map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
