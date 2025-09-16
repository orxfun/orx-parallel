use super::par_collect_into::ParCollectIntoCore;
use crate::Params;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::generic_values::{TransformableValues, Values};
use crate::orch::Orchestrator;
use alloc::vec::Vec;
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

    fn m_collect_into<R, I, M1>(self, orchestrator: R, params: Params, iter: I, map1: M1) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.m_collect_into(orchestrator, params, iter, map1))
    }

    fn x_collect_into<R, I, Vo, X1>(
        self,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(I::Item) -> Vo + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.x_collect_into(orchestrator, params, iter, xap1))
    }

    fn x_try_collect_into<R, I, Vo, X1>(
        self,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    where
        R: Orchestrator,
        I: ConcurrentIter,
        X1: Fn(I::Item) -> Vo + Sync,
        Vo: Values<Item = O>,
    {
        let vec = Vec::from(self);
        vec.x_try_collect_into(orchestrator, params, iter, xap1)
            .map(FixedVec::from)
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
