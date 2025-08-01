use super::par_collect_into::ParCollectIntoCore;
use crate::{
    computations::{M, Values, X, Xfx},
    runner::ParallelRunner,
};
use orx_concurrent_iter::ConcurrentIter;
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
        reserve(&mut vec, iter_len);
        vec
    }

    fn m_collect_into<R, I, M1>(mut self, m: M<I, O, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Send + Sync,
    {
        reserve(&mut self, m.par_len());
        let (_num_spawned, pinned_vec) = m.collect_into::<R, _>(self);
        pinned_vec
    }

    fn x_collect_into<R, I, Vo, M1>(mut self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O> + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Clone + Send + Sync,
    {
        reserve(&mut self, x.par_len());
        let (_num_spawned, pinned_vec) = x.collect_into::<R, _>(self);
        pinned_vec
    }

    fn xfx_collect_into<R, I, Vt, Vo, M1, F, M2>(mut self, mfm: Xfx<I, Vt, Vo, M1, F, M2>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vt: Values + Send + Sync,
        Vt::Item: Send + Sync,
        Vo: Values<Item = O> + Send + Sync,
        M1: Fn(I::Item) -> Vt + Clone + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Clone + Send + Sync,
    {
        reserve(&mut self, mfm.par_len());
        let (_num_spawned, pinned_vec) = mfm.collect_into::<R, _>(self);
        pinned_vec
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}

fn reserve<T, G: GrowthWithConstantTimeAccess>(
    split_vec: &mut SplitVec<T, G>,
    len_to_extend: Option<usize>,
) {
    match len_to_extend {
        None => {
            let capacity_bound = split_vec.capacity_bound();
            split_vec.reserve_maximum_concurrent_capacity(capacity_bound)
        }
        Some(len) => split_vec.reserve_maximum_concurrent_capacity(split_vec.len() + len),
    };
}
