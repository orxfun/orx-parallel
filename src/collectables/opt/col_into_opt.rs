use crate::infallible::Xap;
use crate::option::{ParOpt, ParRunnerOpt};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoOpt<T>: Sized {
    fn opt_col_into<I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParOpt<I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send;

    fn opt_arb_col_into<I, M, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParOpt<I, M, X1, X2, S, R>,
    ) -> Option<Self>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send;
}
