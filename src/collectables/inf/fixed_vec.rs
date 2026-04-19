use crate::collectables::inf::ColIntoInf;
use crate::infallible::{ParIter, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInf<T> for FixedVec<T> {
    fn inf_col_into<I, X, R>(dst: Option<Self>, par: ParIter<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoInf<T>>::inf_col_into(dst, par).into()
    }

    fn inf_arb_col_into<I, X, R>(dst: Option<Self>, par: ParIter<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoInf<T>>::inf_arb_col_into(dst, par).into()
    }
}
