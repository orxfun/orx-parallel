use crate::collectables::Collectable;
use crate::collectables::alg::merge_collected::merge_arb;
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{ParCore, ParIter, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoInf<T>: Sized {
    fn new_empty() -> Self;

    fn inf_col_into<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send;

    fn inf_arb_col_into_from_jagged(dst: &mut Self, thread_collections: Vec<Vec<T>>)
    where
        T: Send;

    fn extend_from_vec(dst: &mut Self, values: Vec<T>);

    fn create_from_vec(values: Vec<T>) -> Self;

    // newcol

    type ThreadColArb: Collectable<T>;

    fn inf_arb_col_into_x<I, X, R>(dst: &mut Self, par: ParIter<I, X, R>)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
        Self: Collectable<T>,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb::<_, _, Self::ThreadColArb>(params, iter, x);
        merge_arb(results, dst);
    }
}
