use crate::collectables::inf_use::ColIntoInfUse;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUse, XapUse};
use crate::infallible_use::{ParUse, XapUse};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInfUse<T> for FixedVec<T> {
    fn inf_use_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoInfUse<T>>::inf_use_col_into(dst, par).into()
    }

    fn inf_use_arb_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoInfUse<T>>::inf_use_arb_col_into(dst, par).into()
    }
}
