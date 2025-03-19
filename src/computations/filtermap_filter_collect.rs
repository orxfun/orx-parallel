use crate::{Maybe, Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct FiltermapFilterCollect<I, MO, O, FilterMap, Filter, P>
where
    I: ConcurrentIter,
    MO: Maybe<O> + Send + Sync,
    O: Send + Sync,
    FilterMap: Fn(I::Item) -> MO + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Filter,
    pinned_vec: P,
    phantom: PhantomData<O>,
}

impl<I, MO, O, FilterMap, Filter, P> FiltermapFilterCollect<I, MO, O, FilterMap, Filter, P>
where
    I: ConcurrentIter,
    MO: Maybe<O> + Send + Sync,
    O: Send + Sync,
    FilterMap: Fn(I::Item) -> MO + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn sequential_fill_bag(mut self) -> P {
        let iter = self.iter.into_seq_iter();
        for maybe in iter.map(self.filter_map) {
            if maybe.has_value() {
                self.pinned_vec.push(maybe.value_unchecked());
            }
        }
        self.pinned_vec
    }
}
