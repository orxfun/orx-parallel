use super::collect_into_core::ParCollectIntoCore;
use crate::{
    fn_sync::FnSync,
    par::{par_filtermap_fil::ParFilterMapFilter, par_flatmap_fil::ParFlatMapFilter},
    ParCollectInto, ParIter, ParMap, ParMapFilter,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use std::fmt::Debug;

impl<O: Default + Send + Sync + Debug> ParCollectInto<O> for Vec<O> {}

impl<O: Default + Send + Sync + Debug> ParCollectIntoCore<O> for Vec<O> {
    type BridgePinnedVec = FixedVec<O>;

    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + FnSync,
    {
        if let Some(iter_len) = par_map.iter_len() {
            self.reserve(iter_len);
        }

        let fixed: FixedVec<_> = self.into();
        let bag: ConcurrentOrderedBag<_, _> = fixed.into();
        par_map.collect_bag(bag).into()
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
                    self.reserve(len);
                    FixedVec::from(self)
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
                    self.reserve(len);
                    FixedVec::from(self)
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
                    self.reserve(len);
                    FixedVec::from(self)
                })
                .into(),
        }
    }

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec> {
        let fixed: FixedVec<_> = self.into();
        fixed.into()
    }

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self {
        bag.into_inner().into()
    }

    fn seq_extend<I: Iterator<Item = O>>(mut self, iter: I) -> Self {
        self.extend(iter);
        self
    }
}
