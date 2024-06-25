use super::par_collect_into::ParCollectInto;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Growth, SplitVec};

impl<O: Default + Send + Sync, G: Growth> ParCollectInto<O> for SplitVec<O, G> {
    type BridgePinnedVec = Self;

    fn map_into<I, M>(mut self, par_map: crate::ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
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

    fn map_filter_into<I, M, F>(mut self, par: crate::ParMapFilter<I, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        if let Some(iter_len) = par.iter_len() {
            let required_len = self.len() + iter_len;
            self.try_reserve_maximum_concurrent_capacity(required_len)
                .expect("Failed to reserve sufficient capacity");
        }

        par.collect_bag(|x| self.push(x));
        self
    }

    fn fmap_filter_into<I, OI, M, F>(
        mut self,
        par: crate::par::par_fmap_fil::ParFMapFilter<I, O, OI, M, F>,
    ) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        if let Some(iter_len) = par.iter_len() {
            let required_len = self.len() + iter_len;
            self.try_reserve_maximum_concurrent_capacity(required_len)
                .expect("Failed to reserve sufficient capacity");
        }

        par.collect_bag(|x| self.push(x));
        self
    }

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec> {
        self.into()
    }

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self {
        bag.into_inner().into()
    }
}
