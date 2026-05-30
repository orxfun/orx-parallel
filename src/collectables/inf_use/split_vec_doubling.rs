use crate::collectables::alg::merge_collected::{
    merge_arb_into_split_vec, merge_ord_into_split_vec,
};
use crate::collectables::inf_use::ColIntoInfUse;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUseCore, ParUseIter, Using, XapUse};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Doubling, SplitVec};

impl<T> ColIntoInfUse<T> for SplitVec<T, Doubling> {
    fn inf_use_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Using,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, u, iter, x);
        merge_ord_into_split_vec(results, dst);
    }

    fn inf_use_arb_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Using,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, u, iter, x);
        merge_arb_into_split_vec(results, dst);
    }
}
