use super::map::ParMap;
use crate::{
    into_par_iter::IntoParIter,
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct Par<I, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
    phantom: PhantomData<R>,
}

impl<I, R> Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    pub(crate) fn new(params: Params, iter: I) -> Self {
        Self {
            iter,
            params,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I) {
        (self.params, self.iter)
    }
}

impl<I> IntoParIter for Par<I, DefaultRunner>
where
    I: ConcurrentIter,
{
    type Item = I::Item;

    fn into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item> {
        self
    }
}

impl<I, R> ParIter<R> for Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    type Item = I::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        &self.iter
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        Par::new(self.params, self.iter)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter) = self.destruct();
        ParMap::new(params, iter, map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (params, iter) = self.destruct();
        C::map_into::<_, _, R>(output, params, iter, no_ops_map)
    }
}

#[inline(always)]
fn no_ops_map<T>(input: T) -> T {
    input
}
