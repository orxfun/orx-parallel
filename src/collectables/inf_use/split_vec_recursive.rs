use crate::collectables::alg::merge_collected::merge_ord_into_split_vec;
use crate::collectables::inf_use::ColIntoInfUse;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, ParUseIterCore, Use, XapUse};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

impl<T> ColIntoInfUse<T> for SplitVec<T, Recursive> {
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
        merge_ord_into_split_vec(results, dst)
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
        let mut dst = dst.unwrap_or_else(|| Self::with_recursive_growth());
        for vec in results {
            dst.append(vec);
        }
        dst
    }
}
