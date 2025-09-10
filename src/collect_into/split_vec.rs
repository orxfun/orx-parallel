use super::par_collect_into::ParCollectIntoCore;
use crate::computational_variants::{ParMap, ParXap};
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::generic_values::{TransformableValues, Values};
use crate::orch::Orchestrator;
use crate::{collect_into::utils::split_vec_reserve, computations::X, runner::ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
#[cfg(test)]
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, PseudoDefault, SplitVec};

impl<O, G> ParCollectIntoCore<O> for SplitVec<O, G>
where
    O: Send + Sync,
    G: GrowthWithConstantTimeAccess,
    Self: PseudoDefault,
{
    type BridgePinnedVec = Self;

    fn empty(iter_len: Option<usize>) -> Self {
        let mut vec = Self::pseudo_default();
        split_vec_reserve(&mut vec, iter_len);
        vec
    }

    fn m_collect_into<R, I, M1>(mut self, m: ParMap<I, O, M1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        split_vec_reserve(&mut self, m.par_len());
        let (_, pinned_vec) = m.par_collect_into(self);
        pinned_vec
    }

    fn x_collect_into<R, I, Vo, X1>(mut self, x: ParXap<I, Vo, X1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(I::Item) -> Vo + Sync,
    {
        split_vec_reserve(&mut self, x.par_len());
        let (_num_spawned, pinned_vec) = x.par_collect_into(self);
        pinned_vec
    }

    fn x_try_collect_into<R, I, Vo, M1>(
        mut self,
        x: X<I, Vo, M1>,
    ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync,
        Self: Sized,
    {
        split_vec_reserve(&mut self, x.par_len());
        let (_num_spawned, result) = x.try_collect_into::<R, _>(self);
        result
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
