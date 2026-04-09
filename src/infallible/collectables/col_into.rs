use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ColIntoInf<T> {
    fn empty(iter_len: Option<usize>) -> Self;

    fn collect_into<I, X, R>(self, par: Par<I, X, R>, exact_len: Option<usize>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send;
}
