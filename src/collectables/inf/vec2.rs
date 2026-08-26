use crate::Vec2;
use crate::collectables::alg::merge_collected::merge_ord_into_vec;
use crate::collectables::inf::ColIntoInf;
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{ParCore, ParIter, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

impl<T> ColIntoInf<T> for Vec2<T> {
    fn new_empty() -> Self {
        Self::from(Vec::new())
    }

    fn inf_col_into<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, iter, x);

        let mut ordered = Vec::new();
        merge_ord_into_vec(results, &mut ordered);
        dst.inner.push(ordered);
    }

    fn inf_arb_col_into<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, iter, x);
        dst.inner.extend(results);
    }

    fn inf_arb_col_into_from_jagged(dst: &mut Self, thread_collections: Vec<Vec<T>>)
    where
        T: Send,
    {
        dst.inner.extend(thread_collections);
    }
}
