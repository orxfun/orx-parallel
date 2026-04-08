use crate::infallible::Xap;
use crate::sizes::{Many, Size};

pub trait SizePairRes: Clone + Copy + Send + Default {
    type S1: Size;

    type S2: Size;

    type ThenBin: SizePairRes<S1 = Self::S1, S2 = <Self::S2 as Size>::ThenBin>;

    type ThenMany: SizePairRes<S1 = Self::S1, S2 = Many>;

    type XapResResult<M, E, X1, X2>: IntoIterator<Item = Result<X2::O, E>>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;
}
