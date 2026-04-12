use crate::collectables::alg::merge_collected::{
    merge_arb_into_split_vec, merge_ord_into_split_vec,
};
use crate::collectables::inf::ColIntoInf;
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Doubling, SplitVec};

impl<T> ColIntoInf<T> for SplitVec<T, Doubling> {
    fn inf_col_into_new<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_new(params, iter, x);
        merge_ord_into_split_vec(results, dst)
    }

    fn inf_arb_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, iter, x);
        merge_arb_into_split_vec(results, dst.unwrap_or_else(|| Self::with_doubling_growth()))
    }
}
