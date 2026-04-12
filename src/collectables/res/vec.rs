use crate::collectables::alg::merge_collected::{
    merge_arb_into_first_vec, merge_arb_into_vec, merge_ord_into,
};
use crate::collectables::res::ColIntoRes;
use crate::infallible::Xap;
use crate::result::{ParRes, ParRunnerRes, SizePairRes};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoRes<T> for Vec<T> {
    fn res_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParRes<I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, iter, x1, x2);

        results.map(|results| {
            let len: usize = results.iter().map(|x| x.len()).sum();
            let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(len));
            dst.reserve(len);
            merge_ord_into(results, FixedVec::from(dst)).into()
        })
    }

    fn res_arb_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParRes<I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, iter, x1, x2);

        results.map(|results| match dst {
            Some(dst) => merge_arb_into_vec(results, dst),
            None => merge_arb_into_first_vec(results),
        })
    }
}
