use core::marker::PhantomData;

use crate::ParIterUsing;
use crate::default_fns::u_map_self;
use crate::generic_values::Vector;
use crate::par_iter_result::IntoResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::ParIterResultUsing;
use crate::using::computational_variants::u_fallible_result::UParResult;
use crate::using::computational_variants::u_map::UParMap;
use crate::using::computational_variants::u_xap::UParXap;
use crate::using::executor::parallel_compute as prc;
use crate::using::using_variants::Using;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator.
pub struct UPar<'using, U, I, R = DefaultRunner>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
{
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    phantom: PhantomData<&'using ()>,
}

impl<'using, U, I, R> UPar<'using, U, I, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
{
    pub(crate) fn new(using: U, orchestrator: R, params: Params, iter: I) -> Self {
        Self {
            using,
            orchestrator,
            params,
            iter,
            phantom: PhantomData,
        }
    }

    pub(crate) fn destruct(self) -> (U, R, Params, I) {
        (self.using, self.orchestrator, self.params, self.iter)
    }
}

unsafe impl<'using, U, I, R> Send for UPar<'using, U, I, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

unsafe impl<'using, U, I, R> Sync for UPar<'using, U, I, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

impl<'using, U, I, R> ParIterUsing<'using, U, R> for UPar<'using, U, I, R>
where
    U: Using<'using>,
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
    ) -> impl ParIterUsing<'using, U, Q, Item = Self::Item> {
        let (using, _, params, iter) = self.destruct();
        UPar::new(using, orchestrator, params, iter)
    }

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<'using, U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        UParMap::new(using, orchestrator, params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<'using, U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        let x1 = move |u: *mut U::Item, i: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            filter(u, &i).then_some(i)
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<'using, U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        let x1 = move |u: *mut U::Item, i: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            Vector(flat_map(u, i))
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<'using, U, R, Item = Out>
    where
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, orchestrator, params, iter) = self.destruct();
        let x1 = move |u: *mut U::Item, i: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            filter_map(u, i)
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(
        self,
    ) -> impl ParIterResultUsing<'using, U, R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
    {
        UParResult::new(self)
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
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
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
