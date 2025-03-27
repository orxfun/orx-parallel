use super::m::M;
use crate::runner::{ComputationKind, ParallelRunner, ParallelTaskWithIdx};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct MReduce<I, O, M1, X>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    X: Fn(O, O) -> O + Send + Sync,
{
    m: M<I, O, M1>,
    reduce: X,
}

impl<I, O, M1, X> MReduce<I, O, M1, X>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    X: Fn(O, O) -> O + Send + Sync,
{
    fn sequential(self) -> Option<O> {
        let (m, reduce) = (self.m, self.reduce);
        let (_, iter, map1) = m.destruct();
        iter.into_seq_iter().map(map1).reduce(reduce)
    }

    fn parallel(self) -> Option<O> {
        todo!()
    }
}
