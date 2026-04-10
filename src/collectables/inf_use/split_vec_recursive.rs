use crate::collectables::inf_use::ColIntoInfUse;
use crate::collectables::utils::merge_ord_into;
use crate::infallible::ParRunnerInfallible;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, XapUse};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

impl<T> ColIntoInfUse<T> for SplitVec<T, Recursive> {
    fn inf_use_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, iter, x);

        let dst = dst.unwrap_or_else(|| Self::with_recursive_growth());
        merge_ord_into(results, dst)
    }

    fn inf_use_arb_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, iter, x);
        let mut dst = dst.unwrap_or_else(|| Self::with_recursive_growth());
        for vec in results {
            dst.append(vec);
        }
        dst
    }
}
