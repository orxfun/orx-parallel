use super::par_collect_into::ParCollectIntoCore;
use crate::computational_variants::ParMap;
use crate::computations::{M, X};
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::orch::Orchestrator;
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
#[cfg(test)]
use orx_pinned_vec::PinnedVec;

impl<O> ParCollectIntoCore<O> for FixedVec<O>
where
    O: Send + Sync,
{
    type BridgePinnedVec = Self;

    fn empty(iter_len: Option<usize>) -> Self {
        let vec = <Vec<_> as ParCollectIntoCore<_>>::empty(iter_len);
        vec.into()
    }

    fn m_collect_into<R, I, M1>(self, m: ParMap<I, O, M1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.m_collect_into::<R, _, _>(m))
    }

    fn x_collect_into<R, I, Vo, M1>(self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O, Fallibility = Infallible>,
        M1: Fn(I::Item) -> Vo + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.x_collect_into::<R, _, _, _>(x))
    }

    fn x_try_collect_into<R, I, Vo, M1>(
        self,
        x: X<I, Vo, M1>,
    ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync,
        Self: Sized,
    {
        let vec = Vec::from(self);
        let result = vec.x_try_collect_into::<R, _, _, _>(x);
        result.map(FixedVec::from)
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
