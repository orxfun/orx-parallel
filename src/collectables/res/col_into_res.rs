use crate::infallible::Xap;
use crate::result::{ParResultIter, ParRunnerRes};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoRes<T>: Sized {
    // fn res_col_into<I, M, E, X1, X2, S, R>(
    //     dst: Option<Self>,
    //     par: ParResultIter<I, M, E, X1, X2, S, R>,
    // ) -> Result<Self, E>
    // where
    //     I: ConcurrentIter,
    //     X1: Xap<I = I::Item, O = Result<M, E>>,
    //     X2: Xap<I = M, O = T>,
    //     S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    //     R: ParRunnerRes,
    //     T: Send,
    //     E: Send;

    // fn res_arb_col_into<I, M, E, X1, X2, S, R>(
    //     dst: Option<Self>,
    //     par: ParResultIter<I, M, E, X1, X2, S, R>,
    // ) -> Result<Self, E>
    // where
    //     I: ConcurrentIter,
    //     X1: Xap<I = I::Item, O = Result<M, E>>,
    //     X2: Xap<I = M, O = T>,
    //     S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    //     R: ParRunnerRes,
    //     T: Send,
    //     E: Send;

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
        E: Send;

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
        E: Send;
}
