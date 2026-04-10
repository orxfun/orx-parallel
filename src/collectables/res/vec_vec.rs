use crate::collectables::res::ColIntoRes;
use crate::collectables::utils::merge_ord_into;
use crate::infallible::Xap;
use crate::result::{ParRes, ParRunnerRes, SizePairRes};
use alloc::vec;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoRes<T> for Vec<Vec<T>> {
    fn res_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParRes<I, M, E, X1, X2, S, R>,
    ) -> Result<Self, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        R: crate::runner::ParRunner,
        T: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect(s, params, iter, x1, x2);

        results.map(|results| {
            let len: usize = results.iter().map(|x| x.len()).sum();

            let mut ordered = Vec::new();
            ordered.reserve(len);
            let ordered = merge_ord_into(results, FixedVec::from(ordered)).into();

            match dst {
                Some(mut lst) => {
                    lst.push(ordered);
                    lst
                }
                None => vec![ordered],
            }
        })
    }
}
