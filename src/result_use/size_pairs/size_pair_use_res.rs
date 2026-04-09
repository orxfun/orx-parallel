use crate::infallible_use::XapUse;
use crate::sizes::SizePair;

pub trait SizePairUseRes: SizePair {
    type XapUseResResult<M, E, X1, X2>: IntoIterator<Item = Result<X2::O, E>>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    fn xap_use_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapUseResResult<M, E, X1, X2>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;
}
