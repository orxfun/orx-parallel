use crate::ParMap;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Growth, SplitVec};

pub trait ParMapCollectInto<O: Send + Sync> {
    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync;
}

impl<O: Send + Sync> ParMapCollectInto<O> for Vec<O> {
    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync,
    {
        if let Some(iter_len) = par_map.iter_len() {
            self.reserve(iter_len);
        }

        let fixed: FixedVec<_> = self.into();
        let bag: ConcurrentOrderedBag<_, _> = fixed.into();
        par_map.collect_bag(bag).into()
    }
}

impl<O: Send + Sync> ParMapCollectInto<O> for FixedVec<O> {
    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync,
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

impl<O: Send + Sync, G: Growth> ParMapCollectInto<O> for SplitVec<O, G> {
    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync,
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
