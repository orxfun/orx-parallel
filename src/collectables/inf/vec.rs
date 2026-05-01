use crate::collectables::alg::merge_collected::{merge_arb_into_vec, merge_ord_into_vec};
use crate::collectables::inf::ColIntoInf;
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{ParCore, ParIter, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

impl<T> ColIntoInf<T> for Vec<T> {
    fn new_empty() -> Self {
        Self::new()
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
        merge_ord_into_vec(results, dst);
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
        merge_arb_into_vec(results, dst);
    }
}
