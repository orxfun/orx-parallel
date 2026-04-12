use crate::collectables::opt::ColIntoOpt;
use crate::collectables::alg::merge_collected::{merge_arb_into_split_vec, merge_ord_into};
use crate::infallible::Xap;
use crate::option::{ParOpt, ParRunnerOpt, SizePairOpt};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Doubling, SplitVec};

impl<T> ColIntoOpt<T> for SplitVec<T, Doubling> {
    fn opt_col_into<I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParOpt<I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, iter, x1, x2);

        results.map(|results| {
            let dst = dst.unwrap_or_else(|| Self::with_doubling_growth());
            merge_ord_into(results, dst)
        })
    }

    fn opt_arb_col_into<I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParOpt<I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, iter, x1, x2);
        results.map(|results| {
            merge_arb_into_split_vec(results, dst.unwrap_or_else(|| Self::with_doubling_growth()))
        })
    }
}
