use crate::collectables::res_use::ColIntoResUse;
use crate::collectables::merge_collected::{merge_arb_into_split_vec, merge_ord_into};
use crate::infallible_use::{Use, XapUse};
use crate::result_use::{ParRunnerUseRes, ParUseRes, SizePairUseRes};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Linear, SplitVec};

impl<T> ColIntoResUse<T> for SplitVec<T, Linear> {
    fn res_use_col_into<U, I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseRes<U, I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, u, iter, x1, x2);

        results.map(|results| {
            let dst = dst.unwrap_or_else(|| SplitVec::with_linear_growth(10));
            merge_ord_into(results, dst)
        })
    }

    fn res_use_arb_col_into<U, I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseRes<U, I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, u, iter, x1, x2);
        results.map(|results| {
            merge_arb_into_split_vec(results, dst.unwrap_or_else(|| Self::with_linear_growth(10)))
        })
    }
}
