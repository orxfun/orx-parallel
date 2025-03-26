use super::par_collect_into::ParCollectIntoCore;
use crate::{
    computations::MapCollect,
    map_filter_map::{Mfm, Values},
    parameters::Params,
    runner::ParallelRunner,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
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

    fn collect_into<R, I, T, Vt, Vo, M1, F, M2>(
        mut self,
        mfm: Mfm<I, T, Vt, O, Vo, M1, F, M2>,
        in_input_order: bool,
    ) -> Self
    where
        R: ParallelRunner,
        I: orx_concurrent_iter::ConcurrentIter,
        Vt: Values<Item = T>,
        O: Send + Sync,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
    {
        reserve(&mut self, mfm.par_len());
        let (_num_spawned, pinned_vec) = mfm.collect_into::<R, _>(in_input_order, self);
        pinned_vec
    }

    fn map_into<I, M, R>(mut self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        R: ParallelRunner,
    {
        reserve(&mut self, iter.try_get_len());
        let bag: ConcurrentOrderedBag<_, _> = self.into();
        let (_num_spawned, pinned_vec) = MapCollect::new(params, iter, map, bag).compute::<R>();
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
        None => split_vec.reserve_maximum_concurrent_capacity(1 << 32),
        Some(len) => split_vec.reserve_maximum_concurrent_capacity(split_vec.len() + len),
    };
}
