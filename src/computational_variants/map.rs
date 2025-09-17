use super::xap::ParXap;
use crate::computational_variants::fallible_result::ParMapResult;
use crate::generic_values::{Vector, WhilstAtom};
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::IntoResult;
use crate::runner::parallel_runner_compute as prc;
use crate::using::{UParMap, UsingClone, UsingFun};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params};
use crate::{ParIterResult, ParIterUsing};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator that maps inputs.
pub struct ParMap<I, O, M1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    orchestrator: R,
    params: Params,
    iter: I,
    map1: M1,
}

impl<I, O, M1, R> ParMap<I, O, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    pub(crate) fn new(orchestrator: R, params: Params, iter: I, map1: M1) -> Self {
        Self {
            orchestrator,
            params,
            iter,
            map1,
        }
    }

    pub(crate) fn destruct(self) -> (R, Params, I, M1) {
        (self.orchestrator, self.params, self.iter, self.map1)
    }
}

unsafe impl<I, O, M1, R> Send for ParMap<I, O, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
}

unsafe impl<I, O, M1, R> Sync for ParMap<I, O, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
}

impl<I, O, M1, R> ParIter<R> for ParMap<I, O, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    type Item = O;

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

    fn with_runner<Q: Orchestrator>(self, orchestrator: Q) -> impl ParIter<Q, Item = Self::Item> {
        let (_, params, iter, map) = self.destruct();
        ParMap::new(orchestrator, params, iter, map)
    }

    // using transformations

    fn using<U, F>(
        self,
        using: F,
    ) -> impl ParIterUsing<UsingFun<F, U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: 'static,
        F: Fn(usize) -> U + Sync,
    {
        let using = UsingFun::new(using);
        let (orchestrator, params, iter, x1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| x1(t);
        UParMap::new(using, orchestrator, params, iter, m1)
    }

    fn using_clone<U>(
        self,
        value: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + 'static,
    {
        let using = UsingClone::new(value);
        let (orchestrator, params, iter, x1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| x1(t);
        UParMap::new(using, orchestrator, params, iter, m1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        let m1 = move |x| map(m1(x));
        ParMap::new(orchestrator, params, iter, m1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();

        let x1 = move |i: I::Item| {
            let value = m1(i);
            filter(&value).then_some(value)
        };
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        let x1 = move |i: I::Item| Vector(flat_map(m1(i)));
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        let x1 = move |i: I::Item| filter_map(m1(i));
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn take_while<While>(self, take_while: While) -> impl ParIter<R, Item = Self::Item>
    where
        While: Fn(&Self::Item) -> bool + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        let x1 = move |value: I::Item| WhilstAtom::new(m1(value), &take_while);
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(self) -> impl ParIterResult<R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
    {
        ParMapResult::new(self)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        output.m_collect_into(orchestrator, params, iter, m1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        prc::reduce::m(orchestrator, params, iter, m1, reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => prc::next::m(orchestrator, params, iter, m1).1,
            IterationOrder::Arbitrary => prc::next_any::m(orchestrator, params, iter, m1).1,
        }
    }
}
