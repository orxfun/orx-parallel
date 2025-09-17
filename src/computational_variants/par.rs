use super::{map::ParMap, xap::ParXap};
use crate::computational_variants::fallible_result::ParResult;
use crate::generic_values::{Vector, WhilstAtom};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::par_iter_result::IntoResult;
use crate::executor::parallel_compute as prc;
use crate::using::{UPar, UsingClone, UsingFun};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params, default_fns::map_self,
};
use crate::{IntoParIter, ParIterResult, ParIterUsing};
use orx_concurrent_iter::chain::ChainKnownLenI;
use orx_concurrent_iter::{ConcurrentIter, ExactSizeConcurrentIter};

/// A parallel iterator.
pub struct Par<I, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    orchestrator: R,
    params: Params,
    iter: I,
}

impl<I, R> Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    pub(crate) fn new(orchestrator: R, params: Params, iter: I) -> Self {
        Self {
            orchestrator,
            iter,
            params,
        }
    }

    pub(crate) fn destruct(self) -> (R, Params, I) {
        (self.orchestrator, self.params, self.iter)
    }
}

unsafe impl<I, R> Send for Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

unsafe impl<I, R> Sync for Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
}

impl<I, R> ParIter<R> for Par<I, R>
where
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

    fn with_runner<Q: ParallelRunner>(self, orchestrator: Q) -> impl ParIter<Q, Item = Self::Item> {
        Par::new(orchestrator, self.params, self.iter)
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
        let (orchestrator, params, iter) = self.destruct();
        UPar::new(using, orchestrator, params, iter)
    }

    fn using_clone<U>(
        self,
        value: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + 'static,
    {
        let using = UsingClone::new(value);
        let (orchestrator, params, iter) = self.destruct();
        UPar::new(using, orchestrator, params, iter)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync,
    {
        let (orchestrator, params, iter) = self.destruct();
        ParMap::new(orchestrator, params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync,
    {
        let (orchestrator, params, iter) = self.destruct();
        let x1 = move |i: Self::Item| filter(&i).then_some(i);
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync,
    {
        let (orchestrator, params, iter) = self.destruct();
        let x1 = move |i: Self::Item| Vector(flat_map(i));
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync,
    {
        let (orchestrator, params, iter) = self.destruct();
        ParXap::new(orchestrator, params, iter, filter_map)
    }

    fn take_while<While>(self, take_while: While) -> impl ParIter<R, Item = Self::Item>
    where
        While: Fn(&Self::Item) -> bool + Sync,
    {
        let (orchestrator, params, iter) = self.destruct();
        let x1 = move |value: Self::Item| WhilstAtom::new(value, &take_while);
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(self) -> impl ParIterResult<R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
    {
        ParResult::new(self)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (orchestrator, params, iter) = self.destruct();
        output.m_collect_into(orchestrator, params, iter, map_self)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter) = self.destruct();
        prc::reduce::m(orchestrator, params, iter, map_self, reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        let (orchestrator, params, iter) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => prc::next::m(orchestrator, params, iter, map_self).1,
            IterationOrder::Arbitrary => prc::next_any::m(orchestrator, params, iter, map_self).1,
        }
    }
}

impl<I, R> Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    /// Creates a chain of this and `other` parallel iterators.
    ///
    /// The first iterator is required to have a known length for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec!['a', 'b', 'c']; // with exact len
    /// let b = vec!['d', 'e', 'f'].into_iter().filter(|x| *x != 'x');
    ///
    /// let chain = a.into_par().chain(b.iter_into_par());
    /// assert_eq!(
    ///     chain.collect::<Vec<_>>(),
    ///     vec!['a', 'b', 'c', 'd', 'e', 'f'],
    /// );
    /// ```
    pub fn chain<C>(self, other: C) -> Par<ChainKnownLenI<I, C::IntoIter>, R>
    where
        I: ExactSizeConcurrentIter,
        C: IntoParIter<Item = I::Item>,
    {
        let (orchestrator, params, iter) = self.destruct();
        let iter = iter.chain(other.into_con_iter());
        Par::new(orchestrator, params, iter)
    }
}
