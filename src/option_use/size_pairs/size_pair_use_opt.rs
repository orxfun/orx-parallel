use crate::infallible_use::XapUse;
use crate::sizes::SizePair;

pub trait SizePairUseOpt: SizePair {
    type XapUseOptResult<M, X1, X2>: IntoIterator<Item = Option<X2::O>>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    fn xap_use_opt<M, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseOptResult<M, X1, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;
}
