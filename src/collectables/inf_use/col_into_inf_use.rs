use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, XapUse};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoInfUse<T>: Sized {
    fn inf_use_col_into<I, X, R>(dst: Option<Self>, par: ParUse<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: XapUse<I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;

    fn inf_use_arb_col_into<I, X, R>(dst: Option<Self>, par: ParUse<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: XapUse<I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;
}
