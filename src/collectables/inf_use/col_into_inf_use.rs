use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, Use, XapUse};
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoInfUse<T>: Sized {
    fn inf_col_into_new<U, I, X, R>(dst: Option<Self>, par: ParUse<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;

    fn inf_use_arb_col_into<U, I, X, R>(dst: Option<Self>, par: ParUse<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;
}
