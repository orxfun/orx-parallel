use crate::infallible::Xap;
use crate::infallible_use::XapUse;
use crate::sizes::{Many, Size};

pub trait SizePair: Clone + Copy + Send + Default {
    type S1: Size;

    type S2: Size;

    type ThenBin: SizePair<S1 = Self::S1, S2 = <Self::S2 as Size>::ThenBin>;

    type ThenMany: SizePair<S1 = Self::S1, S2 = Many>;

    // option

    type XapOptResult<M, X1, X2>: IntoIterator<Item = Option<X2::O>>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    // result

    type XapResResult<M, E, X1, X2>: IntoIterator<Item = Result<X2::O, E>>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    // use - option

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
