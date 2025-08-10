use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, Params,
    computations::{Atom, Vector},
    runner::{DefaultRunner, ParallelRunner},
    using::u_par_iter::ParIterUsing,
    using::{
        Using,
        computational_variants::{u_xap::UParXap, u_xap_filter_xap::UParXapFilterXap},
        computations::{UM, u_map_self_atom},
    },
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that maps inputs.
pub struct UParMap<U, I, O, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    um: UM<U, I, O, M1>,
    phantom: PhantomData<R>,
}

impl<U, I, O, M1, R> UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
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
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
}

unsafe impl<U, I, O, M1, R> Sync for UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
}

impl<U, I, O, M1, R> ParIterUsing<U, R> for UParMap<U, I, O, M1, R>
where
    R: ParallelRunner,
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    type Item = O;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.um.iter()
    }

    fn params(&self) -> Params {
        self.um.params()
    }

    // parameter transformations

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

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterUsing<U, Q, Item = Self::Item> {
        let (using, params, iter, map) = self.destruct();
        UParMap::new(using, params, iter, map)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, params, iter, m1) = self.destruct();
        let m1 = move |u: &mut U::Item, x: I::Item| {
            let v1 = m1(u, x);
            map(u, v1)
        };
        UParMap::new(using, params, iter, m1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, params, iter, m1) = self.destruct();
        let m1 = move |u: &mut U::Item, i: I::Item| Atom(m1(u, i));
        let filter = move |u: &mut U::Item, i: &Self::Item| filter(u, i);
        UParXapFilterXap::new(using, params, iter, m1, filter, u_map_self_atom)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, params, iter, m1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let a = m1(u, i);
            Vector(flat_map(u, a))
        };
        UParXap::new(using, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, params, iter, m1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let a = m1(u, i);
            filter_map(u, a)
        };
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
        Self::Item: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.um.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        self.um.next::<R>().1
    }
}
