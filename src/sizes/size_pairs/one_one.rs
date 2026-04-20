use crate::infallible::{Xap, XapOne};
use crate::infallible_use::{XapUse, XapUseOne};
use crate::sizes::size_pairs::{OneBin, OneMany};
use crate::sizes::{One, size_pair::SizePair};

#[derive(Clone, Copy, Default)]
pub struct OneOne;

impl SizePair for OneOne {
    type S1 = One;

    type S2 = One;

    type ThenBin = OneBin;

    type ThenMany = OneMany;

    // option

    type XapOptResult<M, X1, X2>
        = [Option<X2::O>; 1]
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.one_value(i);
        [a.map(|a| x2.one_value(a))]
    }

    // result

    type XapResResult<M, E, X1, X2>
        = [Result<X2::O, E>; 1]
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let a = x1.one_value(i);
        [a.map(|a| x2.one_value(a))]
    }

    // use - option

    type XapUseOptResult<M, X1, X2>
        = [Option<X2::O>; 1]
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
        let a = x1.one_value(u, i);
        [a.map(|a| x2.one_value(u, a))]
    }

    // use - result

    type XapUseResResult<M, E, X1, X2>
        = [Result<X2::O, E>; 1]
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
        let a = x1.one_value(u, i);
        [a.map(|a| x2.one_value(u, a))]
    }
}
