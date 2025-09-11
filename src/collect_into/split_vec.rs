use super::par_collect_into::ParCollectIntoCore;
use crate::Params;
use crate::collect_into::utils::split_vec_reserve;
use crate::computational_variants::fallible_result::computations::try_collect_into;
use crate::computational_variants::{ParMap, ParXap};
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::generic_values::{TransformableValues, Values};
use crate::orch::Orchestrator;
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

    fn x_try_collect_into<R, I, Vo, X1>(
        mut self,
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
        Self: Sized,
    {
        let par_len = match (params.is_sequential(), iter.try_get_len()) {
            (true, _) => None, // not required to concurrent reserve when seq
            (false, x) => x,
        };

        split_vec_reserve(&mut self, par_len);
        let (_num_spawned, result) = try_collect_into(orchestrator, params, iter, xap1, self);
        result
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
