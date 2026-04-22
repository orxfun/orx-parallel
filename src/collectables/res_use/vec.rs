use crate::collectables::alg::merge_collected::{
    merge_arb_into_first_vec, merge_arb_into_vec, merge_ord_into_vec,
};
use crate::collectables::res_use::ColIntoResUse;
use crate::infallible_use::{Use, XapUse};
use crate::result_use::{ParRunnerUseRes, ParUseResultIter, ParUseResultCore};
use crate::sizes::SizePair;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

impl<T> ColIntoResUse<T> for Vec<T> {
    fn res_use_col_into<U, I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, u, iter, x1, x2);

        results.map(|results| merge_ord_into_vec(results, dst))
    }

    fn res_use_arb_col_into<U, I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, u, iter, x1, x2);

        results.map(|results| match dst {
            Some(dst) => merge_arb_into_vec(results, dst),
            None => merge_arb_into_first_vec(results),
        })
    }
}
