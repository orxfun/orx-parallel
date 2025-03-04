use crate::{
    collect_into::ParCollectInto,
    parameters::{ChunkSize, NumThreads},
};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParIter: Sized {
    type Item: Send + Sync;

    fn con_iter(&self) -> &impl ConcurrentIter;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone;

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
