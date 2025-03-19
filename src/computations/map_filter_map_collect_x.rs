use crate::{
    parameters::Params,
    runner::{ComputationKind, ParallelRunner},
    Maybe,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

use super::default_fns::{filter_true, map_maybe_value, map_self, maybe_has_value};

pub struct MapFilterMapCollect<I, T, O, Map1, Filter, Map2, P>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T,
    Filter: Fn(&T) -> bool,
    Map2: Fn(T) -> O,
    P: IntoConcurrentPinnedVec<O>,
{
    params: Params,
    iter: I,
    map1: Map1,
    filter: Filter,
    map2: Map2,
    bag: ConcurrentOrderedBag<O, P>,
}

impl<I, T, O, Map1, Filter, Map2, P> MapFilterMapCollect<I, T, O, Map1, Filter, Map2, P>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T,
    Filter: Fn(&T) -> bool,
    Map2: Fn(T) -> O,
    P: IntoConcurrentPinnedVec<O>,
{
    pub fn new(
        bag: ConcurrentOrderedBag<O, P>,
        params: Params,
        iter: I,
        map1: Map1,
        filter: Filter,
        map2: Map2,
    ) -> Self {
        Self {
            params,
            iter,
            map1,
            filter,
            map2,
            bag,
        }
    }
}

pub fn map_filter_map<I, T, O, Map1, Filter, Map2, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    map1: Map1,
    filter: Filter,
    map2: Map2,
) -> MapFilterMapCollect<I, T, O, Map1, Filter, Map2, P>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T,
    Filter: Fn(&T) -> bool,
    Map2: Fn(T) -> O,
    P: IntoConcurrentPinnedVec<O>,
{
    MapFilterMapCollect::new(bag, params, iter, map1, filter, map2)
}

pub fn map_filter<I, O, Map1, Filter, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    map1: Map1,
    filter: Filter,
) -> MapFilterMapCollect<I, O, O, Map1, Filter, impl Fn(O) -> O, P>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> O,
    Filter: Fn(&O) -> bool,
    P: IntoConcurrentPinnedVec<O>,
{
    MapFilterMapCollect::new(bag, params, iter, map1, filter, map_self)
}

pub fn filter_map<I, O, Filter, Map2, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    filter: Filter,
    map2: Map2,
) -> MapFilterMapCollect<I, I::Item, O, impl Fn(I::Item) -> I::Item, Filter, Map2, P>
where
    I: ConcurrentIter,
    Filter: Fn(&I::Item) -> bool,
    Map2: Fn(I::Item) -> O,
    P: IntoConcurrentPinnedVec<O>,
{
    MapFilterMapCollect::new(bag, params, iter, map_self, filter, map2)
}

pub fn filter<I, Filter, P>(
    bag: ConcurrentOrderedBag<I::Item, P>,
    params: Params,
    iter: I,
    filter: Filter,
) -> MapFilterMapCollect<
    I,
    I::Item,
    I::Item,
    impl Fn(I::Item) -> I::Item,
    Filter,
    impl Fn(I::Item) -> I::Item,
    P,
>
where
    I: ConcurrentIter,
    Filter: Fn(&I::Item) -> bool,
    P: IntoConcurrentPinnedVec<I::Item>,
{
    MapFilterMapCollect::new(bag, params, iter, map_self, filter, map_self)
}

// filtermap

pub fn filtermap<I, O, M, FilterMap, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    fmap: FilterMap,
) -> MapFilterMapCollect<I, M, O, FilterMap, impl Fn(&M) -> bool, impl Fn(M) -> O, P>
where
    I: ConcurrentIter,
    M: Maybe<O>,
    FilterMap: Fn(I::Item) -> M,
    P: IntoConcurrentPinnedVec<O>,
{
    let filter = maybe_has_value::<O, M>;
    let map2 = map_maybe_value::<O, M>;
    MapFilterMapCollect::new(bag, params, iter, fmap, filter, map2)
}

pub fn filtermap_filter<I, O, M, FilterMap, Filter, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    fmap: FilterMap,
    filter: Filter,
) -> MapFilterMapCollect<I, M, O, FilterMap, impl Fn(&M) -> bool, impl Fn(M) -> O, P>
where
    I: ConcurrentIter,
    M: Maybe<O>,
    FilterMap: Fn(I::Item) -> M,
    Filter: Fn(&O) -> bool,
    P: IntoConcurrentPinnedVec<O>,
{
    let filter = move |maybe: &M| maybe.has_value_and(&filter);
    let map2 = map_maybe_value::<O, M>;
    MapFilterMapCollect::new(bag, params, iter, fmap, filter, map2)
}

pub fn map_filtermap<I, T, O, M, Map, FilterMap, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    map: Map,
    fmap: FilterMap,
) -> MapFilterMapCollect<I, M, O, impl Fn(I::Item) -> M, impl Fn(&M) -> bool, impl Fn(M) -> O, P>
where
    I: ConcurrentIter,
    M: Maybe<O>,
    Map: Fn(I::Item) -> T,
    FilterMap: Fn(T) -> M,
    P: IntoConcurrentPinnedVec<O>,
{
    let fmap = move |input: I::Item| fmap(map(input));
    filtermap(bag, params, iter, fmap)
}

pub fn filtermap_map<I, T, O, M, FilterMap, Map, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    fmap: FilterMap,
    map: Map,
) -> MapFilterMapCollect<I, M, O, FilterMap, impl Fn(&M) -> bool, impl Fn(M) -> O, P>
where
    I: ConcurrentIter,
    M: Maybe<T>,
    FilterMap: Fn(I::Item) -> M,
    Map: Fn(T) -> O,
    P: IntoConcurrentPinnedVec<O>,
{
    let filter = maybe_has_value::<T, M>;
    let map2 = move |maybe: M| map(maybe.unwrap());
    MapFilterMapCollect::new(bag, params, iter, fmap, filter, map2)
}

pub fn filter_filtermap<I, O, M, Filter, FilterMap, P>(
    bag: ConcurrentOrderedBag<O, P>,
    params: Params,
    iter: I,
    filter: Filter,
    fmap: FilterMap,
) -> MapFilterMapCollect<
    I,
    Option<O>,
    O,
    impl Fn(I::Item) -> Option<O>,
    impl Fn(&Option<O>) -> bool,
    impl Fn(Option<O>) -> O,
    P,
>
where
    I: ConcurrentIter,
    M: Maybe<O>,
    Filter: Fn(&I::Item) -> bool,
    FilterMap: Fn(I::Item) -> M,
    P: IntoConcurrentPinnedVec<O>,
{
    let map1 = move |input: I::Item| match filter(&input) {
        true => fmap(input).into_option(),
        false => None,
    };
    let filter = maybe_has_value::<O, Option<O>>;
    let map2 = map_maybe_value::<O, Option<O>>;
    MapFilterMapCollect::new(bag, params, iter, map1, filter, map2)
}
