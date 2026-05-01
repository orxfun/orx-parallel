use crate::infallible::{ParIter, Xap};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoInf<T>: Sized {
    fn new_empty() -> Self;

    fn inf_col_into_new<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send;

    fn inf_arb_col_into_new<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send;
}
