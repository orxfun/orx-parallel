use super::xap_filter_xap::ParXapFilterXap;
use crate::{
    computations::{map_self_atom, Atom, M},
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, CollectOrdering, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParMap<I, O, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync + Clone,
{
    m: M<I, O, M1>,
    phantom: PhantomData<R>,
}

impl<I, O, M1, R> ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync + Clone,
{
    pub(crate) fn new(params: Params, iter: I, m: M1) -> Self {
        Self {
            m: M::new(params, iter, m),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1) {
        self.m.destruct()
    }
}

impl<I, O, M1, R> ParIter<R> for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync + Clone,
{
    type Item = O;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.m.iter()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.m.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.m.chunk_size(chunk_size);
        self
    }

    fn collect_ordering(mut self, collect: CollectOrdering) -> Self {
        self.m.collect_ordering(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (params, iter, map) = self.destruct();
        ParMap::new(params, iter, map)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter, map1) = self.destruct();
        let map = move |x| map(map1(x));
        ParMap::new(params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (params, iter, map1) = self.destruct();
        let map1 = move |i: I::Item| Atom(map1(i));
        ParXapFilterXap::new(params, iter, map1, filter, map_self_atom)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.m_collect_into::<R, _, _>(self.m)
    }
}
