use crate::infallible_use::{Use, XapUse};
use crate::option_use::{ParRunnerUseOpt, ParUseOptionIter};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoOptUse<T>: Sized {
    // fn opt_use_col_into_new<U, I, M, X1, X2, S, R>(
    //     dst: &mut Self,
    //     par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    // ) -> Option<()>
    // where
    //     U: Use,
    //     I: ConcurrentIter,
    //     X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    //     X2: XapUse<U = U::Item, I = M, O = T>,
    //     S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    //     R: ParRunnerUseOpt,
    //     T: Send;

    // fn opt_use_arb_col_into_new<U, I, M, X1, X2, S, R>(
    //     dst: &mut Self,
    //     par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    // ) -> Option<()>
    // where
    //     U: Use,
    //     I: ConcurrentIter,
    //     X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    //     X2: XapUse<U = U::Item, I = M, O = T>,
    //     S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    //     R: ParRunnerUseOpt,
    //     T: Send;

    fn opt_use_col_into<U, I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send;

    fn opt_use_arb_col_into<U, I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send;
}
