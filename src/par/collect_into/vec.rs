use super::collect_into_core::ParCollectIntoCore;
use crate::{
    core::{
        filtermap_fil_col::{par_filtermap_fil_col_vec, seq_filtermap_fil_col_vec},
        flatmap_fil_col::{par_flatmap_fil_col_vec, seq_flatmap_fil_col_vec},
        map_col::map_col,
        map_fil_col::{par_map_fil_col_vec, seq_map_fil_col_vec},
    },
    par::{
        par_filtermap_fil::ParFilterMapFilter, par_flatmap_fil::ParFlatMapFilter, par_map::ParMap,
        par_map_fil::ParMapFilter,
    },
    ParCollectInto,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::fmt::Debug;

impl<O: Send + Sync + Debug> ParCollectInto<O> for Vec<O> {}

impl<O: Send + Sync + Debug> ParCollectIntoCore<O> for Vec<O> {
    type BridgePinnedVec = FixedVec<O>;

    fn map_into<I, M>(mut self, par_map: ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        match par_map.iter_len() {
            None => SplitVec::with_doubling_growth_and_fragments_capacity(32)
                .map_into(par_map)
                .to_vec(),
            Some(iter_len) => {
                self.reserve(iter_len);
                let fixed: FixedVec<_> = self.into();
                let bag: ConcurrentOrderedBag<_, _> = fixed.into();
                let (params, iter, map) = par_map.destruct();
                map_col(params, iter, map, bag).into()
            }
        }
    }

    fn map_filter_into<I, M, F>(mut self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        let (params, iter, map, filter) = par.destruct();

        match params.is_sequential() {
            true => seq_map_fil_col_vec(iter, map, filter, &mut self),
            false => par_map_fil_col_vec(params, iter, map, filter, &mut self),
        }
        self
    }

    fn flatmap_filter_into<I, OI, M, F>(mut self, par: ParFlatMapFilter<I, O, OI, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, flat_map, filter) = par.destruct();

        match params.is_sequential() {
            true => seq_flatmap_fil_col_vec(iter, flat_map, filter, &mut self),
            false => par_flatmap_fil_col_vec(params, iter, flat_map, filter, &mut self),
        }
        self
    }

    fn filtermap_filter_into<I, FO, M, F>(mut self, par: ParFilterMapFilter<I, FO, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        FO: crate::Fallible<O> + Send + Sync + Debug,
        M: Fn(I::Item) -> FO + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        let (params, iter, filter_map, filter) = par.destruct();

        match params.is_sequential() {
            true => seq_filtermap_fil_col_vec(iter, filter_map, filter, &mut self),
            false => par_filtermap_fil_col_vec(params, iter, filter_map, filter, &mut self),
        }
        self
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
