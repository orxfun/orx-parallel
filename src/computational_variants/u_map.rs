use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params,
    computational_variants::{u_xap::UParXap, u_xap_filter_xap::UParXapFilterXap},
    computations::{Atom, UM, Using, Vector, u_map_self_atom},
    runner::{DefaultRunner, ParallelRunner},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that maps inputs.
pub struct UParMap<U, I, O, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync + Clone,
{
    um: UM<U, I, O, M1>,
    phantom: PhantomData<R>,
}

impl<U, I, O, M1, R> UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync + Clone,
{
    pub(crate) fn new(using: U, params: Params, iter: I, m1: M1) -> Self {
        Self {
            um: UM::new(using, params, iter, m1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (U, Params, I, M1) {
        self.um.destruct()
    }
}

unsafe impl<U, I, O, M1, R> Send for UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync + Clone,
{
}

unsafe impl<U, I, O, M1, R> Sync for UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync + Clone,
{
}

impl<U, I, O, M1, R> ParIter<R> for UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync + Clone,
{
    type Item = O;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.um.iter()
    }

    fn params(&self) -> Params {
        self.um.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.um.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.um.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.um.iteration_order(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (using, params, iter, map) = self.destruct();
        UParMap::new(using, params, iter, map)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (using, params, iter, m1) = self.destruct();
        let m1 = move |u: &mut U::Item, x: I::Item| map(m1(u, x));
        UParMap::new(using, params, iter, m1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (using, params, iter, m1) = self.destruct();
        let m1 = move |u: &mut U::Item, i: I::Item| Atom(m1(u, i));
        let filter = move |_: &mut U::Item, i: &Self::Item| filter(i);
        UParXapFilterXap::new(using, params, iter, m1, filter, u_map_self_atom)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync,
    {
        let (using, params, iter, m1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| Vector(flat_map(m1(u, i)));
        UParXap::new(using, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (using, params, iter, m1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| filter_map(m1(u, i));
        UParXap::new(using, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.u_m_collect_into::<R, _, _, _>(self.um)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        self.um.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        self.um.next()
    }
}
