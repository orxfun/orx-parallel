use crate::collectables::alg::merge_collected::{
    merge_arb_into_split_vec, merge_ord_into_split_vec,
};
use crate::collectables::opt::ColIntoOpt;
use crate::infallible::Xap;
use crate::option::{ParOptionCore, ParOptionIter, ParRunnerOpt};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Linear, SplitVec};

impl<T> ColIntoOpt<T> for SplitVec<T, Linear> {
    fn opt_col_into<I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParOptionIter<I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, iter, x1, x2);

        results.map(|results| merge_ord_into_split_vec(results, dst))
    }

    fn opt_arb_col_into<I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParOptionIter<I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, iter, x1, x2);
        results.map(|results| merge_arb_into_split_vec(results, dst))
    }
}
