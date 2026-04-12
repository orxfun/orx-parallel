use crate::collectables::opt_use::ColIntoOptUse;
use crate::collectables::alg::merge_collected::{merge_arb_into_first_vec, merge_arb_into_vec, merge_ord_into};
use crate::infallible_use::{Use, XapUse};
use crate::option_use::{ParRunnerUseOpt, ParUseOpt, SizePairUseOpt};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoOptUse<T> for Vec<T> {
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
            let len: usize = results.iter().map(|x| x.len()).sum();
            let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(len));
            dst.reserve(len);
            merge_ord_into(results, FixedVec::from(dst)).into()
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

        results.map(|results| match dst {
            Some(dst) => merge_arb_into_vec(results, dst),
            None => merge_arb_into_first_vec(results),
        })
    }
}
