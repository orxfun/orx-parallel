use crate::infallible_use::{XapUse, XapUseBin, XapUseOne};
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseRes;
use crate::sizes::BinOne;

impl SizePairUseRes for BinOne {
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
        let a = x1.bin_value(u, i);
        a.map(|a| a.map(|a| x2.one_value(u, a)))
    }
}
