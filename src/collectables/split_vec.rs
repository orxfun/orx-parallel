use crate::collectables::col_into_inf::ColIntoInf;
use crate::collectables::utils::merge_collected_into;
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, PseudoDefault, SplitVec};

impl<T, G: GrowthWithConstantTimeAccess> ColIntoInf<T> for SplitVec<T, G> {
    fn inf_collect_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, iter, x);

        let dst = dst.unwrap_or_else(|| SplitVec::pseudo_default());
        merge_collected_into(results, dst)
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
        let (iter, x, mut exe, params) = par.destruct();
        let mut dst = dst.unwrap_or_else(|| SplitVec::pseudo_default());

        match exact_len {
            Some(len) => {
                dst.reserve_maximum_concurrent_capacity(dst.len() + len);
            }
            None => {
                // TODO: collect_into might be faster
                let capacity_bound = dst.capacity_bound();
                dst.reserve_maximum_concurrent_capacity(capacity_bound);
            }
        }
        exe.collect_arbitrary(params, iter, x, dst)
    }
}
