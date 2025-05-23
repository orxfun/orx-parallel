use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params,
    computational_variants::xap_filter_xap::ParXapFilterXap,
    computations::{Values, X, map_self_atom},
    runner::{DefaultRunner, ParallelRunner},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct ParXap<I, Vo, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    x: X<I, Vo, M1>,
    phantom: PhantomData<R>,
}

impl<I, Vo, M1, R> ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    pub(crate) fn new(params: Params, iter: I, x1: M1) -> Self {
        Self {
            x: X::new(params, iter, x1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1) {
        self.x.destruct()
    }
}

unsafe impl<I, Vo, M1, R> Send for ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
}

unsafe impl<I, Vo, M1, R> Sync for ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
}

impl<I, Vo, M1, R> ParIter<R> for ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.x.iter()
    }

    fn params(&self) -> &Params {
        self.x.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.x.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.x.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.x.iteration_order(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (params, iter, map1) = self.destruct();
        ParXap::new(params, iter, map1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.map(map.clone())
        };

        ParXap::new(params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (params, iter, x1) = self.destruct();
        ParXapFilterXap::new(params, iter, x1, filter, map_self_atom)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.flat_map(flat_map.clone())
        };
        ParXap::new(params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.flat_map(filter_map.clone())
        };
        ParXap::new(params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.x_collect_into::<R, _, _, _>(self.x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        self.x.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        self.x.next()
    }
}
