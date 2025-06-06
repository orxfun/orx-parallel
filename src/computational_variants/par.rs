use super::{map::ParMap, xap::ParXap, xap_filter_xap::ParXapFilterXap};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params,
    computations::{M, Vector, map_self, map_self_atom},
    runner::{DefaultRunner, ParallelRunner},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
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

    fn m(self) -> M<I, I::Item, impl Fn(I::Item) -> I::Item> {
        let (params, iter) = self.destruct();
        M::new(params, iter, map_self)
    }
}

unsafe impl<I, R> Send for Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

unsafe impl<I, R> Sync for Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
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

    fn params(&self) -> &Params {
        &self.params
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

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.params = self.params.with_collect_ordering(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        Par::new(self.params, self.iter)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter) = self.destruct();
        ParMap::new(params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (params, iter) = self.destruct();
        ParXapFilterXap::new(params, iter, map_self_atom, filter, map_self_atom)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync,
    {
        let (params, iter) = self.destruct();
        let x1 = move |i: Self::Item| Vector(flat_map(i)); // TODO: inline
        ParXap::new(params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (params, iter) = self.destruct();
        ParXap::new(params, iter, filter_map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.m_collect_into::<R, _, _>(self.m())
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        self.m().reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        self.m().next()
    }
}
