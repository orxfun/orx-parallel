use crate::infallible::Xap;
use crate::result::{ParRes, SizePairRes};
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoRes<T>: Sized {
    fn inf_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParRes<I, M, E, X1, X2, S, R>,
    ) -> Self
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunner,
        T: Send;

    fn inf_arb_col_into<I, M, E, X1, X2, S, R>(
        dst: Option<Self>,
        par: ParRes<I, M, E, X1, X2, S, R>,
        exact_len: Option<usize>,
    ) -> Self
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunner,
        T: Send;
}
