use super::collect_into_core::ParCollectIntoCore;
use crate::{
    fn_sync::FnSync,
    par::{
        par_filtermap_fil::ParFilterMapFilter, par_flatmap_fil::ParFlatMapFilter, par_map::ParMap,
        par_map_fil::ParMapFilter,
    },
    ParCollectInto, ParIter,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_split_vec::*;
use std::fmt::Debug;

impl<O: Send + Sync + Debug, G: Growth> ParCollectInto<O> for SplitVec<O, G> {}

impl<O: Send + Sync + Debug, G: Growth> ParCollectIntoCore<O> for SplitVec<O, G> {
    type BridgePinnedVec = Self;

    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + FnSync,
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
        M: Fn(I::Item) -> O + FnSync,
        F: Fn(&O) -> bool + FnSync,
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

    fn flatmap_filter_into<I, OI, M, F>(mut self, par: ParFlatMapFilter<I, O, OI, M, F>) -> Self
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

    fn filtermap_filter_into<I, FO, M, F>(mut self, par: ParFilterMapFilter<I, FO, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        FO: crate::Fallible<O> + Send + Sync + Debug,
        M: Fn(I::Item) -> FO + FnSync,
        F: Fn(&O) -> bool + FnSync,
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

    fn seq_extend<I: Iterator<Item = O>>(mut self, iter: I) -> Self {
        for x in iter {
            self.push(x)
        }
        self
    }
}
