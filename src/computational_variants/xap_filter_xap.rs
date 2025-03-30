use crate::{
    computations::{Values, Vector, Xfx},
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, CollectOrdering, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParXapFilterXap<I, Vt, Vo, M1, F, M2, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vt::Item: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    phantom: PhantomData<R>,
}

impl<I, Vt, Vo, M1, F, M2, R> ParXapFilterXap<I, Vt, Vo, M1, F, M2, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vt::Item: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub(crate) fn new(params: Params, iter: I, x1: M1, f: F, x2: M2) -> Self {
        Self {
            xfx: Xfx::new(params, iter, x1, f, x2),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1, F, M2) {
        self.xfx.destruct()
    }
}

impl<I, Vt, Vo, M1, F, M2, R> ParIter<R> for ParXapFilterXap<I, Vt, Vo, M1, F, M2, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vt::Item: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.xfx.iter()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.xfx.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.xfx.chunk_size(chunk_size);
        self
    }

    fn collect_ordering(mut self, collect: CollectOrdering) -> Self {
        self.xfx.collect_ordering(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (params, iter, map1, filter, map2) = self.destruct();
        ParXapFilterXap::new(params, iter, map1, filter, map2)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter, x1, f, x2) = self.destruct();
        let x2 = move |t: Vt::Item| {
            let vo = x2(t);
            vo.map(map.clone())
        };

        ParXapFilterXap::new(params, iter, x1, f, x2)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, x1, f, x2) = self.destruct();
        let x2 = move |t: Vt::Item| {
            let vo = x2(t);
            vo.filter(filter.clone())
        };

        ParXapFilterXap::new(params, iter, x1, f, x2)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync + Clone,
    {
        let (params, iter, x1, f, x2) = self.destruct();
        let x2 = move |t: Vt::Item| {
            let vo = x2(t);
            vo.flat_map(flat_map.clone())
        };

        ParXapFilterXap::new(params, iter, x1, f, x2)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (params, iter, x1, f, x2) = self.destruct();
        let x2 = move |t: Vt::Item| {
            let vo = x2(t);
            vo.filter_map(filter_map.clone())
        };

        ParXapFilterXap::new(params, iter, x1, f, x2)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.xfx_collect_into::<R, _, _, _, _, _, _>(self.xfx)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        None
    }
}
