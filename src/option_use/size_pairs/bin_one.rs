use crate::infallible_use::{XapUse, XapUseBin, XapUseOne};
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseOpt;
use crate::sizes::BinOne;

impl SizePairUseOpt for BinOne {
    type XapUseOptResult<M, X1, X2>
        = Option<Option<X2::O>>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_use_opt<M, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseOptResult<M, X1, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>,
    {
        let a = x1.bin_value(u, i);
        a.map(|a| a.map(|a| x2.one_value(u, a)))
    }
}
