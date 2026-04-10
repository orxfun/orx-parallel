use crate::collectables::opt::ColIntoOpt;
use crate::infallible::Xap;
use crate::option::{ParOpt, ParRunnerOpt, SizePairOpt};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoOpt<T> for FixedVec<T> {
    fn opt_col_into<I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParOpt<I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoOpt<T>>::opt_col_into(dst, par).map(|v| v.into())
    }

    fn opt_arb_col_into<I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParOpt<I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
    {
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoOpt<T>>::opt_arb_col_into(dst, par).map(|v| v.into())
    }
}
