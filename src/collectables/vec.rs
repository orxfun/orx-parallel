use crate::collectables::col_into_inf::ColIntoInf;
use crate::collectables::par_col_into::ParCollectInto;
use crate::collectables::utils::{extend_vec_from_split, merge_collected_into};
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

impl<T> ParCollectInto<T> for Vec<T> {}

impl<T> ColIntoInf<T> for Vec<T> {
    fn inf_collect_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, iter, x);
        let len: usize = results.iter().map(|x| x.len()).sum();

        let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(len));
        dst.reserve(len);
        merge_collected_into(results, FixedVec::from(dst)).into()
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

        match exact_len {
            Some(len) => {
                let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(len));
                dst.reserve(len);
                exe.collect_arbitrary(params, iter, x, FixedVec::from(dst))
                    .into_inner()
            }
            None => {
                // TODO: collect_into might be faster
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = exe.collect_arbitrary(params, iter, x, split_vec);
                let dst = dst.unwrap_or_else(|| Vec::with_capacity(split_vec.len()));
                extend_vec_from_split(dst, split_vec)
            }
        }
    }
}
