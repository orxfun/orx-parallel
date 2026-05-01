use crate::infallible_use::{Use, XapUse};
use crate::result_use::{ParRunnerUseRes, ParUseResultIter};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoResUse<T>: Sized {
    fn res_use_col_into_new<U, I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send;

    fn res_use_arb_col_into_new<U, I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send;

    fn res_use_col_into<U, I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send;

    fn res_use_arb_col_into<U, I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send;
}
