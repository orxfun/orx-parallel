use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, Params,
    computations::Vector,
    runner::{DefaultRunner, ParallelRunner},
    using::u_par_iter::ParIterUsing,
    using::{
        Using,
        computational_variants::{
            u_map::UParMap, u_xap::UParXap, u_xap_filter_xap::UParXapFilterXap,
        },
        computations::{UM, u_map_self, u_map_self_atom},
    },
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct UPar<U, I, R = DefaultRunner>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
{
    using: U,
    iter: I,
    params: Params,
    phantom: PhantomData<R>,
}

impl<U, I, R> UPar<U, I, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
{
    pub(crate) fn new(using: U, params: Params, iter: I) -> Self {
        Self {
            using,
            iter,
            params,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (U, Params, I) {
        (self.using, self.params, self.iter)
    }

    #[allow(clippy::type_complexity)]
    fn u_m(self) -> UM<U, I, I::Item, impl Fn(&mut U::Item, I::Item) -> I::Item> {
        let (using, params, iter) = self.destruct();
        UM::new(using, params, iter, u_map_self)
    }
}

unsafe impl<U, I, R> Send for UPar<U, I, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

unsafe impl<U, I, R> Sync for UPar<U, I, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

impl<U, I, R> ParIterUsing<U, R> for UPar<U, I, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
{
    type Item = I::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        &self.iter
    }

    fn params(&self) -> Params {
        self.params
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

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterUsing<U, Q, Item = Self::Item> {
        UPar::new(self.using, self.params, self.iter)
    }

    // computational transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(&mut <U as Using>::Item, Self::Item) -> Out + Send + Sync + Clone,
    {
        let (using, params, iter) = self.destruct();
        let map = move |u: &mut U::Item, x: Self::Item| map(u, x);
        UParMap::new(using, params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Send + Sync + Clone,
    {
        let (using, params, iter) = self.destruct();
        let filter = move |u: &mut U::Item, x: &Self::Item| filter(u, x);
        UParXapFilterXap::new(
            using,
            params,
            iter,
            u_map_self_atom,
            filter,
            u_map_self_atom,
        )
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Send + Sync + Clone,
    {
        let (using, params, iter) = self.destruct();
        let x1 = move |u: &mut U::Item, i: Self::Item| Vector(flat_map(u, i));
        UParXap::new(using, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(&mut <U as Using>::Item, Self::Item) -> Option<Out> + Send + Sync + Clone,
    {
        let (using, params, iter) = self.destruct();
        let x1 = move |u: &mut U::Item, x: Self::Item| filter_map(u, x);
        UParXap::new(using, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.u_m_collect_into::<R, _, _, _>(self.u_m())
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        self.u_m().reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        self.u_m().next()
    }
}
