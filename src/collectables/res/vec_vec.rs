use crate::collectables::alg::merge_collected::merge_ord_into_vec;
use crate::collectables::res::ColIntoRes;
use crate::infallible::Xap;
use crate::result::{ParRes, ParResIterCore, ParRunnerRes};
use crate::sizes::SizePair;
use alloc::vec;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

impl<T> ColIntoRes<T> for Vec<Vec<T>> {
    fn res_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParRes<I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: crate::runner::ParRunner,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, iter, x1, x2);

        results.map(|results| {
            let ordered = merge_ord_into_vec(results, None);
            match dst {
                Some(mut lst) => {
                    lst.push(ordered);
                    lst
                }
                None => vec![ordered],
            }
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb(s, params, iter, x1, x2);

        results.map(|results| match dst {
            Some(mut lst) => {
                lst.extend(results);
                lst
            }
            None => results,
        })
    }
}
