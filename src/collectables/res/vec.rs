use crate::collectables::alg::merge_collected::{merge_arb_into_vec, merge_ord_into_vec};
use crate::collectables::res::ColIntoRes;
use crate::infallible::Xap;
use crate::result::{ParResultCore, ParResultIter, ParRunnerRes};
use crate::sizes::SizePair;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

impl<T> ColIntoRes<T> for Vec<T> {
    fn res_col_into<I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParResultIter<I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, iter, x1, x2);

        results.map(|results| merge_ord_into_vec(results, dst))
    }

    fn res_arb_col_into<I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParResultIter<I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, iter, x1, x2);

        results.map(|results| merge_arb_into_vec(results, dst))
    }
}
