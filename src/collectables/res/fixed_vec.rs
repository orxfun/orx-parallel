use crate::collectables::res::ColIntoRes;
use crate::infallible::Xap;
use crate::result::{ParResultIter, ParRunnerRes};
use crate::sizes::SizePair;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoRes<T> for FixedVec<T> {
    fn res_col_into_new<I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParResultIter<I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoRes<T>>::res_col_into_new(dst, par)
    }

    fn res_arb_col_into_new<I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParResultIter<I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerRes,
        T: Send,
        E: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoRes<T>>::res_arb_col_into_new(dst, par)
    }

    fn res_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParResultIter<I, M, E, X1, X2, S, R>,
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
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoRes<T>>::res_col_into(dst, par).map(|v| v.into())
    }

    fn res_arb_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParResultIter<I, M, E, X1, X2, S, R>,
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
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoRes<T>>::res_arb_col_into(dst, par).map(|v| v.into())
    }
}
