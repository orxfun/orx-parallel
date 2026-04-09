use crate::infallible_use::{XapUse, XapUseBin};
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseRes;
use crate::sizes::BinBin;

impl SizePairUseRes for BinBin {
    type XapUseResResult<M, X1, X2>
        = Option<Option<X2::O>>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_use_res<M, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseResResult<M, X1, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>,
    {
        x1.bin_value(u, i).and_then(|a| match a {
            Some(a) => x2.bin_value(u, a).map(Some),
            None => Some(None),
        })
    }
}
