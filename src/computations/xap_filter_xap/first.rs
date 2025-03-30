use super::xfx::Xfx;
use crate::computations::heap_sort::heap_sort_into;
use crate::computations::Values;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTask};
use crate::CollectOrdering;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct XfxFirst<I, Vt, Vo, M1, F, M2>(Xfx<I, Vt, Vo, M1, F, M2>)
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync;

impl<I, Vt, Vo, M1, F, M2> XfxFirst<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn first<R>(xfx: Xfx<I, Vt, Vo, M1, F, M2>) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let xfx_first = XfxFirst(xfx);
        (0, None)
    }

    fn sequential(self) -> Option<Vo::Item> {
        let (_, iter, xap1, filter, xap2) = self.0.destruct();

        iter.into_seq_iter()
            .flat_map(|i| xap1(i).values().into_iter())
            .filter(|t| filter(t))
            .flat_map(|t| xap2(t).values().into_iter())
            .next()
    }
}
