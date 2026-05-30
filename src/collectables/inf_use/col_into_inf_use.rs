use crate::infallible_use::{ParRunnerInfallibleUse, ParUseIter, Using, XapUse};
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoInfUse<T>: Sized {
    fn inf_use_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Using,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;

    fn inf_use_arb_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Using,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;
}
