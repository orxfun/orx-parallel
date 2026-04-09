use crate::infallible_use::{XapBin, XapOne, XapUse};
use crate::result_use::size_pairs::size_pair_use_res::SizePairUseRes;
use crate::sizes::OneBin;

impl SizePairUseRes for OneBin {
    type XapUseResResult<M, E, X1, X2>
        = Option<Result<X2::O, E>>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_use_res<M, E, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseResResult<M, E, X1, X2>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>,
    {
        match x1.one_value(u, i) {
            Ok(a) => x2.bin_value(u, a).map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
