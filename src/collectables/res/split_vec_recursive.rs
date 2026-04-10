use crate::collectables::res::ColIntoRes;
use crate::collectables::utils::merge_ord_into;
use crate::infallible::Xap;
use crate::result::{ParRes, ParRunnerRes, SizePairRes};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

impl<T> ColIntoRes<T> for SplitVec<T, Recursive> {
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
            let dst = dst.unwrap_or_else(|| SplitVec::with_recursive_growth());
            merge_ord_into(results, dst)
        })
    }
}
