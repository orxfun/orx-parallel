use crate::infallible::collectables::col_into::ColIntoInf;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInf<T> for FixedVec<T> {
    fn empty(exact_len: Option<usize>) -> Self {
        <Vec<T> as ColIntoInf<T>>::empty(exact_len).into()
    }

    fn collect_into<I, X, R>(self, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        <Vec<T> as ColIntoInf<T>>::collect_into(self.into(), par).into()
    }

    fn collect_arbitrary_into<I, X, R>(self, par: Par<I, X, R>, exact_len: Option<usize>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        <Vec<T> as ColIntoInf<T>>::collect_arbitrary_into(self.into(), par, exact_len).into()
    }
}
