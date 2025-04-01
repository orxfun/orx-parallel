use crate::{
    collect_into::ParCollectInto,
    parameters::{ChunkSize, CollectOrdering, NumThreads},
    runner::{DefaultRunner, ParallelRunner},
};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParIter<R = DefaultRunner>: Sized
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    fn con_iter(&self) -> &impl ConcurrentIter;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn collect_ordering(self, collect: CollectOrdering) -> Self;

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item>;

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone;

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync + Clone;

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync + Clone;

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync;

    // early exit

    fn next(self) -> Option<Self::Item>;

    fn get_any(self) -> Option<Self::Item>;

    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(predicate).next()
    }

    fn find_any<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(predicate).get_any()
    }
}
