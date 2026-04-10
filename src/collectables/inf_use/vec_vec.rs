use crate::collectables::inf_use::ColIntoInfUse;
use crate::collectables::utils::merge_ord_into;
use crate::infallible::ParRunnerInfallible;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, Use, XapUse};
use alloc::vec;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInfUse<T> for Vec<Vec<T>> {
    fn inf_use_col_into<U, I, X, R>(dst: Option<Self>, par: ParUse<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, u, iter, x);
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

    fn inf_use_arb_col_into<U, I, X, R>(dst: Option<Self>, par: ParUse<U, I, X, R>) -> Self
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, u, iter, x);
        match dst {
            Some(mut lst) => {
                lst.extend(results);
                lst
            }
            None => results,
        }
    }
}
