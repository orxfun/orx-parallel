use crate::collectables::opt_use::ColIntoOptUse;
use crate::collectables::utils::merge_ord_into;
use crate::infallible_use::{Use, XapUse};
use crate::option_use::{ParRunnerUseOpt, ParUseOpt, SizePairUseOpt};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

impl<T> ColIntoOptUse<T> for SplitVec<T, Recursive> {
    fn opt_use_col_into<U, I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseOpt<U, I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, u, iter, x1, x2);

        results.map(|results| {
            let dst = dst.unwrap_or_else(|| Self::with_recursive_growth());
            merge_ord_into(results, dst)
        })
    }

    fn opt_use_arb_col_into<U, I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseOpt<U, I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, u, iter, x1, x2);
        let mut dst = dst.unwrap_or_else(|| Self::with_recursive_growth());
        results.map(|results| {
            for vec in results {
                dst.append(vec);
            }
            dst
        })
    }
}
