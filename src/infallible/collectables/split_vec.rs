use crate::infallible::collectables::col_into::ColIntoInf;
use crate::infallible::collectables::utils::{
    extend_vec_from_split, merge_collected_into, split_vec_reserve,
};
use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_split_vec::{Growth, GrowthWithConstantTimeAccess, PseudoDefault, SplitVec};

impl<T, G: Growth> ColIntoInf<T> for SplitVec<T, G> {
    fn collect_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
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

    fn collect_arbitrary_into<I, X, R>(
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
        todo!()
    }
    //     fn empty(exact_len: Option<usize>) -> Self {
    //         let mut vec = Self::pseudo_default();
    //         split_vec_reserve(&mut vec, exact_len);
    //         vec
    //     }

    //     fn collect_into<I, X, R>(mut self, par: Par<I, X, R>) -> Self
    //     where
    //         I: ConcurrentIter,
    //         X: Xap<I = I::Item, O = T>,
    //         R: ParRunner,
    //         T: Send,
    //     {
    //         let (iter, x, mut exe, params) = par.destruct();
    //         let results = exe.collect(params, iter, x);
    //         let len: usize = results.iter().map(|x| x.len()).sum();

    //         split_vec_reserve(&mut self, Some(len));
    //         todo!()
    //     }

    //     fn collect_arbitrary_into<I, X, R>(self, par: Par<I, X, R>, exact_len: Option<usize>) -> Self
    //     where
    //         I: ConcurrentIter,
    //         X: Xap<I = I::Item, O = T>,
    //         R: ParRunner,
    //         T: Send,
    //     {
    //         todo!()
    //     }
}
