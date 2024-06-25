use super::par_collect_into::ParCollectInto;
use crate::{par::par_fmap_fil::ParFMapFilter, ParIter, ParMap, ParMapFilter};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Growth, SplitVec};
use std::fmt::Debug;

impl<O: Default + Send + Sync + Debug, G: Growth> ParCollectInto<O> for SplitVec<O, G> {
    type BridgePinnedVec = Self;

    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        if let Some(len) = par_map.iter_len() {
            self.try_reserve_maximum_concurrent_capacity(self.len() + len)
                .expect("Failed to reserve sufficient capacity");
        }

        let bag: ConcurrentOrderedBag<_, _> = self.into();
        par_map.collect_bag(bag)
    }

    fn map_filter_into<I, M, F>(mut self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        match par.params().is_sequential() {
            true => par.collect_bag_seq(self, |v, x| v.push(x)),
            false => par
                .collect_bag_par(|len| {
                    unsafe { self.grow_to(self.len() + len, false) }
                        .expect("Failed to reserve sufficient capacity");
                    self
                })
                .into(),
        }

        // if let Some(iter_len) = par.iter_len() {
        //     let required_len = self.len() + iter_len;
        //     self.try_reserve_maximum_concurrent_capacity(required_len)
        //         .expect("Failed to reserve sufficient capacity");
        // }

        // par.collect_bag_zzz(|x| self.push(x));
        // self
    }

    fn fmap_filter_into<I, OI, M, F>(mut self, par: ParFMapFilter<I, O, OI, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        match par.params().is_sequential() {
            true => par.collect_bag_seq(self, |v, x| v.push(x)),
            false => par
                .collect_bag_par(|len| {
                    unsafe { self.grow_to(self.len() + len, false) }
                        .expect("Failed to reserve sufficient capacity");
                    self
                })
                .into(),
        }
    }

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec> {
        self.into()
    }

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self {
        bag.into_inner().into()
    }
}
