use super::collect_into_core::ParCollectIntoCore;
use crate::{
    par::{par_filtermap_fil::ParFilterMapFilter, par_flatmap_fil::ParFlatMapFilter},
    ParCollectInto, ParIter, ParMap, ParMapFilter,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::FixedVec;
use std::fmt::Debug;

impl<O: Default + Send + Sync + Debug> ParCollectInto<O> for FixedVec<O> {}

impl<O: Default + Send + Sync + Debug> ParCollectIntoCore<O> for FixedVec<O> {
    type BridgePinnedVec = Self;

    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        let vec: Vec<_> = self.into();
        vec.map_into(par_map).into()
    }

    fn map_filter_into<I, M, F>(self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        match par.params().is_sequential() {
            true => par
                .collect_bag_seq(Vec::from(self), |v, x| v.push(x))
                .into(),
            false => par
                .collect_bag_par(|len| {
                    let mut vec: Vec<_> = self.into();
                    vec.reserve(len);
                    FixedVec::from(vec)
                })
                .into(),
        }
    }

    fn flatmap_filter_into<I, OI, M, F>(self, par: ParFlatMapFilter<I, O, OI, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        match par.params().is_sequential() {
            true => par
                .collect_bag_seq(Vec::from(self), |v, x| v.push(x))
                .into(),
            false => par
                .collect_bag_par(|len| {
                    let mut vec: Vec<_> = self.into();
                    vec.reserve(len);
                    FixedVec::from(vec)
                })
                .into(),
        }
    }

    fn filtermap_filter_into<I, FO, M, F>(self, par: ParFilterMapFilter<I, FO, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        FO: crate::Fallible<O> + Send + Sync + Debug,
        M: Fn(I::Item) -> FO + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        match par.params().is_sequential() {
            true => par
                .collect_bag_seq(Vec::from(self), |v, x| v.push(x))
                .into(),
            false => par
                .collect_bag_par(|len| {
                    let mut vec: Vec<_> = self.into();
                    vec.reserve(len);
                    FixedVec::from(vec)
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

    fn seq_extend<I: Iterator<Item = O>>(self, iter: I) -> Self {
        let mut vec = self.into_inner();
        vec.extend(iter);
        vec.into()
    }
}
