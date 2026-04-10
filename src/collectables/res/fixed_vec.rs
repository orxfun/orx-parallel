use crate::collectables::res::ColIntoRes;
use crate::infallible::Xap;
use crate::result::{ParRes, ParRunnerRes, SizePairRes};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoRes<T> for FixedVec<T> {
    fn inf_col_into<I, M, E, X1, X2, S, R>(
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
        let dst = dst.map(|x| x.into_inner());
        <Vec<T> as ColIntoRes<T>>::inf_col_into(dst, par).map(|v| v.into())
    }
}
