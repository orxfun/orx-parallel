use crate::collectables::inf_use::ColIntoInfUse;
use crate::collectables::alg::merge_collected::{merge_arb_into_split_vec, merge_ord_into};
use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, Use, XapUse};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Linear, SplitVec};

impl<T> ColIntoInfUse<T> for SplitVec<T, Linear> {
    fn inf_use_col_into<U, I, X, R>(dst: Option<Self>, par: ParUse<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, u, iter, x);

        let dst = dst.unwrap_or_else(|| Self::with_linear_growth(10));
        merge_ord_into(results, dst)
    }

    fn inf_use_arb_col_into<U, I, X, R>(dst: Option<Self>, par: ParUse<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, u, iter, x);
        merge_arb_into_split_vec(results, dst.unwrap_or_else(|| Self::with_linear_growth(10)))
    }
}
