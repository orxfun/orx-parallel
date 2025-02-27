use crate::{
    computations::{MapCollect, ParallelRunner},
    parameters::Params,
};

use super::par_collect_into::ParCollectIntoCore;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

impl<T, G> ParCollectIntoCore<T> for SplitVec<T, G>
where
    T: Send + Sync,
    G: GrowthWithConstantTimeAccess,
{
    type BridgePinnedVec = Self;

    fn map_into<I, M, R>(mut self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunner,
    {
        match iter.try_get_len() {
            None => self.reserve_maximum_concurrent_capacity(1 << 32),
            Some(len) => self.reserve_maximum_concurrent_capacity(self.len() + len),
        };

        let bag: ConcurrentOrderedBag<_, _> = self.into();

        let (_num_spawned, bag) = MapCollect::new(params, iter, map, bag).compute::<R>();

        unsafe { bag.into_inner().unwrap_only_if_counts_match() }
    }
}
