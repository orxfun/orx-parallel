use crate::ParMap;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Growth, SplitVec};

/// Collections into which parallel map computations can be collected.
pub trait ParMapCollectInto<O: Default + Send + Sync> {
    /// Performs the parallel map operation, collecting the results into this collection.
    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone;
}

impl<O: Default + Send + Sync> ParMapCollectInto<O> for Vec<O> {
    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        if let Some(iter_len) = par_map.iter_len() {
            self.reserve(iter_len);
        }

        let fixed: FixedVec<_> = self.into();
        let bag: ConcurrentOrderedBag<_, _> = fixed.into();
        par_map.collect_bag(bag).into()
    }
}

impl<O: Default + Send + Sync> ParMapCollectInto<O> for FixedVec<O> {
    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        let mut vec: Vec<_> = self.into();

        if let Some(iter_len) = par_map.iter_len() {
            vec.reserve(iter_len);
        }

        let fixed: FixedVec<_> = vec.into();
        let bag: ConcurrentOrderedBag<_, _> = fixed.into();
        par_map.collect_bag(bag)
    }
}

impl<O: Default + Send + Sync, G: Growth> ParMapCollectInto<O> for SplitVec<O, G> {
    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        if let Some(iter_len) = par_map.iter_len() {
            let required_len = self.len() + iter_len;
            self.try_reserve_maximum_concurrent_capacity(required_len)
                .expect("Failed to reserve sufficient capacity");
        }

        let bag: ConcurrentOrderedBag<_, _> = self.into();
        par_map.collect_bag(bag)
    }
}
