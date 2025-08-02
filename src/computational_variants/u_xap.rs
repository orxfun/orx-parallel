use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params,
    computational_variants::u_xap_filter_xap::UParXapFilterXap,
    computations::{UX, Using, Values, X, u_map_self_atom},
    runner::{DefaultRunner, ParallelRunner},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct UParXap<U, I, Vo, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
    ux: UX<U, I, Vo, M1>,
    phantom: PhantomData<R>,
}

impl<U, I, Vo, M1, R> UParXap<U, I, Vo, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
    pub(crate) fn new(using: U, params: Params, iter: I, x1: M1) -> Self {
        Self {
            ux: UX::new(using, params, iter, x1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (U, Params, I, M1) {
        self.ux.destruct()
    }
}

unsafe impl<U, I, Vo, M1, R> Send for UParXap<U, I, Vo, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
}

unsafe impl<U, I, Vo, M1, R> Sync for UParXap<U, I, Vo, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
}

impl<U, I, Vo, M1, R> ParIter<R> for UParXap<U, I, Vo, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.ux.iter()
    }

    fn params(&self) -> Params {
        self.ux.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.ux.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.ux.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.ux.iteration_order(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (using, params, iter, map1) = self.destruct();
        UParXap::new(using, params, iter, map1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.map(map.clone())
        };

        UParXap::new(using, params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (using, params, iter, x1) = self.destruct();
        let filter = move |_: &mut U::Item, x: &Self::Item| filter(x);
        UParXapFilterXap::new(using, params, iter, x1, filter, u_map_self_atom)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.flat_map(flat_map.clone())
        };
        UParXap::new(using, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.flat_map(filter_map.clone())
        };
        UParXap::new(using, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.u_x_collect_into::<R, _, _, _, _>(self.ux)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        self.ux.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        self.ux.next()
    }
}
