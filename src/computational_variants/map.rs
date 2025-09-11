use super::xap::ParXap;
use crate::ParIterResult;
use crate::computational_variants::fallible_result::ParMapResult;
use crate::generic_values::{Vector, WhilstAtom};
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::IntoResult;
use crate::runner::parallel_runner_compute;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    using::{UsingClone, UsingFun, computational_variants::UParMap},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

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

    pub(crate) fn par_collect_into<P>(self, pinned_vec: P) -> (usize, P)
    where
        P: IntoConcurrentPinnedVec<O>,
        O: Send,
    {
        match (self.params.is_sequential(), self.params.iteration_order) {
            (true, _) => (0, self.seq_collect_into(pinned_vec)),
            #[cfg(test)]
            (false, IterationOrder::Arbitrary) => {
                let (orchestrator, params, iter, m1) = self.destruct();
                parallel_runner_compute::collect_arbitrary::m(
                    orchestrator,
                    params,
                    iter,
                    m1,
                    pinned_vec,
                )
            }
            (false, _) => {
                let (orchestrator, params, iter, m1) = self.destruct();
                parallel_runner_compute::collect_ordered::m(
                    orchestrator,
                    params,
                    iter,
                    m1,
                    pinned_vec,
                )
            }
        }
    }

    fn seq_collect_into<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<O>,
    {
        let (_, _, iter, map1) = self.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(i));
        }

        pinned_vec
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
        U: Send + 'static,
        F: FnMut(usize) -> U,
    {
        let using = UsingFun::new(using);
        let (orchestrator, params, iter, m1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| m1(t);
        UParMap::new(using, params, iter, m1)
    }

    fn using_clone<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send + 'static,
    {
        let using = UsingClone::new(using);
        let (orchestrator, params, iter, m1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| m1(t);
        UParMap::new(using, params, iter, m1)
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
        output.m_collect_into(self)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        parallel_runner_compute::reduce::m(orchestrator, params, iter, m1, reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        let (orchestrator, params, iter, m1) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => {
                parallel_runner_compute::next::m(orchestrator, params, iter, m1).1
            }
            IterationOrder::Arbitrary => {
                parallel_runner_compute::next_any::m(orchestrator, params, iter, m1).1
            }
        }
    }
}
