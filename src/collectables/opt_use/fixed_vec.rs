use crate::collectables::opt_use::ColIntoOptUse;
use crate::infallible_use::{Use, XapUse};
use crate::option_use::{ParRunnerUseOpt, ParUseOptionIter};
use crate::sizes::SizePair;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoOptUse<T> for FixedVec<T> {
    fn opt_use_col_into_new<U, I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoOptUse<T>>::opt_use_col_into_new(dst, par)
    }

    fn opt_use_arb_col_into_new<U, I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send,
    {
        let dst = dst.as_mut_vec();
        <Vec<T> as ColIntoOptUse<T>>::opt_use_arb_col_into_new(dst, par)
    }
}
