use super::xfx::Xfx;
use crate::computations::heap_sort::heap_sort_into;
use crate::computations::Values;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTask};
use crate::CollectOrdering;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct XfxCollect<I, Vt, Vo, M1, F, M2, X>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    reduce: X,
}

impl<I, Vt, Vo, M1, F, M2, X> XfxCollect<I, Vt, Vo, M1, F, M2, X>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    fn sequential(self) -> Option<Vo::Item> {
        let (xfx, reduce) = (self.xfx, self.reduce);
        let (_, iter, xap1, filter, xap2) = xfx.destruct();

        iter.into_seq_iter()
            .filter_map(|x| xap1(x).fx_reduce(None, &filter, &xap2, &reduce))
            .reduce(&reduce)
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<Vo::Item>) {
        let (xfx, reduce) = (self.xfx, self.reduce);
        let (params, iter, xap1, filter, xap2) = xfx.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        runner.xfx_reduce(&iter, &xap1, &filter, &xap2, &reduce)
    }
}
