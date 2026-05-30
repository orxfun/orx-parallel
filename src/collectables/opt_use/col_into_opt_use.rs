use crate::infallible_use::{Using, XapUse};
use crate::option_use::{ParRunnerUseOpt, ParUseOptionIter};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoOptUse<T>: Sized {
    fn opt_use_col_into<U, I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        U: Using,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send;

    fn opt_use_arb_col_into<U, I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        U: Using,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send;
}
