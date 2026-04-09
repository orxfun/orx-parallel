use crate::collectables::col_into_inf::ColIntoInf;
use crate::collectables::par_col_into::ParCollectInto;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ParCollectInto<T> for FixedVec<T> {}

impl<T> ColIntoInf<T> for FixedVec<T> {
    fn inf_collect_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoInf<T>>::inf_collect_into(dst, par).into()
    }

    fn inf_collect_arbitrary_into<I, X, R>(
        dst: Option<Self>,
        par: Par<I, X, R>,
        exact_len: Option<usize>,
    ) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoInf<T>>::inf_collect_arbitrary_into(dst, par, exact_len).into()
    }
}
