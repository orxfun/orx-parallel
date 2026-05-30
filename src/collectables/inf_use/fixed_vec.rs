use crate::collectables::inf_use::ColIntoInfUse;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUseIter, Using, XapUse};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInfUse<T> for FixedVec<T> {
    fn inf_use_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Using,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoInfUse<T>>::inf_use_col_into(dst, par);
    }

    fn inf_use_arb_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Using,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoInfUse<T>>::inf_use_arb_col_into(dst, par);
    }
}
