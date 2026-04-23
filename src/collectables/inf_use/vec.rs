use crate::collectables::alg::merge_collected::{
    merge_arb_into_first_vec, merge_arb_into_vec, merge_ord_into_vec,
};
use crate::collectables::inf_use::ColIntoInfUse;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUseIter, ParUseCore, Use, XapUse};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

impl<T> ColIntoInfUse<T> for Vec<T> {
    fn inf_use_col_into<U, I, X, R>(dst: Option<Self>, par: ParUseIter<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, u, iter, x);
        merge_ord_into_vec(results, dst)
    }

    fn inf_use_arb_col_into<U, I, X, R>(dst: Option<Self>, par: ParUseIter<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, u, iter, x);

        match dst {
            Some(dst) => merge_arb_into_vec(results, dst),
            None => merge_arb_into_first_vec(results),
        }
    }
}
