use crate::collectables::alg::merge_collected::{
    merge_arb_into_split_vec_new, merge_ord_into_split_vec, merge_ord_into_split_vec_new,
};
use crate::collectables::inf_use::ColIntoInfUse;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUseCore, ParUseIter, Use, XapUse};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

impl<T> ColIntoInfUse<T> for SplitVec<T, Recursive> {
    fn inf_use_col_into_new<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, u, iter, x);
        merge_ord_into_split_vec_new(results, dst);
    }

    fn inf_use_arb_col_into_new<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, u, iter, x);
        merge_arb_into_split_vec_new(results, dst);
    }
}
