use super::{collect::MCollect, m::M, reduce::MReduce};
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        MCollect::compute::<R>(self, pinned_vec)
    }

    pub fn reduce<R, X>(self, reduce: X) -> (usize, Option<O>)
    where
        R: ParallelRunner,
        X: Fn(O, O) -> O + Send + Sync,
    {
        MReduce::compute::<R>(self, reduce)
    }
}
