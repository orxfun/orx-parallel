use crate::infallible::Xap;
use crate::sizes::SizePair;

pub trait SizePairOpt: SizePair {
    type XapOptResult<M, X1, X2>: IntoIterator<Item = Option<X2::O>>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;
}
