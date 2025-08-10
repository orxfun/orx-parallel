use super::par_collect_into::ParCollectIntoCore;
use crate::{
    collect_into::utils::split_vec_reserve,
    computations::{M, Values, X},
    runner::ParallelRunner,
};
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

    fn m_collect_into<R, I, M1>(mut self, m: M<I, O, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        split_vec_reserve(&mut self, m.par_len());
        let (_num_spawned, pinned_vec) = m.collect_into::<R, _>(self);
        pinned_vec
    }

    fn x_collect_into<R, I, Vo, M1>(mut self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync,
    {
        split_vec_reserve(&mut self, x.par_len());
        let (_num_spawned, pinned_vec) = x.collect_into::<R, _>(self);
        pinned_vec
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
