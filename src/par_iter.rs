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
}
