use crate::collectables::inf::ColIntoInf;
use crate::infallible::{ParIter, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInf<T> for FixedVec<T> {
    fn new_empty() -> Self {
        Self::new(0)
    }

    fn inf_col_into<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoInf<T>>::inf_col_into(dst, par);
    }

    fn inf_arb_col_into<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoInf<T>>::inf_arb_col_into(dst, par);
    }

    fn inf_arb_col_into_from_jagged(dst: &mut Self, thread_collections: Vec<Vec<T>>)
    where
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoInf<T>>::inf_arb_col_into_from_jagged(dst, thread_collections);
    }

    fn extend_from_vec(dst: &mut Self, values: Vec<T>) {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoInf<T>>::extend_from_vec(dst, values);
    }

    fn create_from_vec(values: Vec<T>) -> Self {
        values.into()
    }
}
