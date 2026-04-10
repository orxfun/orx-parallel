use crate::collectables::inf::ColIntoInf;
use crate::collectables::utils::merge_ord_into;
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use alloc::vec;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInf<T> for Vec<Vec<T>> {
    fn inf_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, iter, x);
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
    }

    fn inf_arb_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, iter, x);
        match dst {
            Some(mut lst) => {
                lst.extend(results);
                lst
            }
            None => results,
        }
    }
}
