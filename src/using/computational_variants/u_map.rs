use crate::ParIterUsing;
use crate::generic_values::Vector;
use crate::par_iter_result::IntoResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::ParIterResultUsing;
use crate::using::computational_variants::u_fallible_result::UParMapResult;
use crate::using::computational_variants::u_xap::UParXap;
use crate::using::executor::parallel_compute as prc;
use crate::using::using_variants::Using;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator that maps inputs.
pub struct UParMap<U, I, O, M1, R = DefaultRunner>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    map1: M1,
}

impl<U, I, O, M1, R> UParMap<U, I, O, M1, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    pub(crate) fn new(using: U, orchestrator: R, params: Params, iter: I, map1: M1) -> Self {
        Self {
            using,
            orchestrator,
            params,
            iter,
            map1,
        }
    }

    pub(crate) fn destruct(self) -> (U, R, Params, I, M1) {
        (
            self.using,
            self.orchestrator,
            self.params,
            self.iter,
            self.map1,
        )
    }
}

unsafe impl<U, I, O, M1, R> Send for UParMap<U, I, O, M1, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
}

unsafe impl<U, I, O, M1, R> Sync for UParMap<U, I, O, M1, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
}

impl<U, I, O, M1, R> ParIterUsing<U, R> for UParMap<U, I, O, M1, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    type Item = O;

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

    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterUsing<U, Q, Item = Self::Item> {
        let (using, _, params, iter, x1) = self.destruct();
        UParMap::new(using, orchestrator, params, iter, x1)
    }

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();
        let m1 = move |u: &mut U::Item, x: I::Item| {
            let v1 = m1(u, x);
            map(u, v1)
        };
        UParMap::new(using, orchestrator, params, iter, m1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();

        let x1 = move |u: &mut U::Item, i: I::Item| {
            let value = m1(u, i);
            filter(u, &value).then_some(value)
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let a = m1(u, i);
            Vector(flat_map(u, a))
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let a = m1(u, i);
            filter_map(u, a)
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(self) -> impl ParIterResultUsing<U, R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
    {
        UParMapResult::new(self)
    }

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();
        output.u_m_collect_into(using, orchestrator, params, iter, m1)
    }

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();
        prc::reduce::m(using, orchestrator, params, iter, m1, reduce).1
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        let (using, orchestrator, params, iter, m1) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => prc::next::m(using, orchestrator, params, iter, m1).1,
            IterationOrder::Arbitrary => prc::next_any::m(using, orchestrator, params, iter, m1).1,
        }
    }
}
