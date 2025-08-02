use super::{xap::ParXap, xap_filter_xap::ParXapFilterXap};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    computational_variants::u_map::UParMap,
    computations::{Atom, M, UsingClone, Vector, map_self_atom, using_clone},
    runner::{DefaultRunner, ParallelRunner},
    u_par_iter::IntoParIterUsing,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that maps inputs.
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
    pub(crate) fn new(params: Params, iter: I, m1: M1) -> Self {
        Self {
            m: M::new(params, iter, m1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1) {
        self.m.destruct()
    }
}

unsafe impl<I, O, M1, R> Send for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync + Clone,
{
}

unsafe impl<I, O, M1, R> Sync for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync + Clone,
{
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

    fn params(&self) -> Params {
        self.m.params()
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

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.m.iteration_order(collect);
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
        let (params, iter, m1) = self.destruct();
        let m1 = move |x| map(m1(x));
        ParMap::new(params, iter, m1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (params, iter, m1) = self.destruct();
        let m1 = move |i: I::Item| Atom(m1(i));
        ParXapFilterXap::new(params, iter, m1, filter, map_self_atom)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync,
    {
        let (params, iter, m1) = self.destruct();
        let x1 = move |i: I::Item| Vector(flat_map(m1(i)));
        ParXap::new(params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (params, iter, m1) = self.destruct();
        let x1 = move |i: I::Item| filter_map(m1(i));
        ParXap::new(params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.m_collect_into::<R, _, _>(self.m)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        self.m.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        self.m.next()
    }
}

impl<I, O, M1, R> IntoParIterUsing<R> for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync + Clone,
{
    fn using<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send,
    {
        let using = using_clone(using);
        let (params, iter, m1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| m1(t);
        UParMap::new(using, params, iter, m1)
    }
}
