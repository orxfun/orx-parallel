use crate::ParIterUsing;
use crate::default_fns::u_map_self;
use crate::generic_values::Vector;
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::using::computational_variants::u_map::UParMap;
use crate::using::computational_variants::u_xap::UParXap;
use crate::using::runner::parallel_runner_compute as prc;
use crate::using::using_variants::Using;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator.
pub struct UPar<U, I, R = DefaultOrchestrator>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
{
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
}

impl<U, I, R> UPar<U, I, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
{
    pub(crate) fn new(using: U, orchestrator: R, params: Params, iter: I) -> Self {
        Self {
            using,
            orchestrator,
            params,
            iter,
        }
    }

    pub(crate) fn destruct(self) -> (U, R, Params, I) {
        (self.using, self.orchestrator, self.params, self.iter)
    }
}

unsafe impl<U, I, R> Send for UPar<U, I, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
{
}

unsafe impl<U, I, R> Sync for UPar<U, I, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
{
}

impl<U, I, R> ParIterUsing<U, R> for UPar<U, I, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
{
    type Item = I::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        &self.iter
    }

    fn params(&self) -> Params {
        self.params
    }

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

    fn with_runner<Q: Orchestrator>(
        self,
        orchestrator: Q,
    ) -> impl ParIterUsing<U, Q, Item = Self::Item> {
        let (using, _, params, iter) = self.destruct();
        UPar::new(using, orchestrator, params, iter)
    }

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Map: Fn(&mut <U as Using>::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        UParMap::new(using, orchestrator, params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut <U as Using>::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        let x1 = move |u: &mut U::Item, i: Self::Item| filter(u, &i).then_some(i);
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut <U as Using>::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        let x1 = move |u: &mut U::Item, i: Self::Item| Vector(flat_map(u, i));
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        FilterMap: Fn(&mut <U as Using>::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        UParXap::new(using, orchestrator, params, iter, filter_map)
    }

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        output.u_m_collect_into(using, orchestrator, params, iter, u_map_self)
    }

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(&mut <U as Using>::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        prc::reduce::m(using, orchestrator, params, iter, u_map_self, reduce).1
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => {
                prc::next::m(using, orchestrator, params, iter, u_map_self).1
            }
            IterationOrder::Arbitrary => {
                prc::next_any::m(using, orchestrator, params, iter, u_map_self).1
            }
        }
    }
}
