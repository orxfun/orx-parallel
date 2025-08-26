use super::{map::ParMap, xap::ParXap};
use crate::ParIterResult;
use crate::computational_variants::fallible_result::ParResult;
use crate::generic_values::{Vector, WhilstAtom};
use crate::par_iter_result::IntoResult;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    computations::{M, map_self},
    runner::{DefaultRunner, ParallelRunner},
    using::{UsingClone, UsingFun, computational_variants::UPar},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct Par<I, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
    phantom: PhantomData<R>,
}

impl<I, R> Par<I, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
{
    pub(crate) fn new(params: Params, iter: I) -> Self {
        Self {
            iter,
            params,
            phantom: PhantomData,
        }
    }

    pub(crate) fn destruct(self) -> (Params, I) {
        (self.params, self.iter)
    }

    fn m(self) -> M<I, I::Item, impl Fn(I::Item) -> I::Item> {
        let (params, iter) = self.destruct();
        M::new(params, iter, map_self)
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

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        Par::new(self.params, self.iter)
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
        UPar::new(using, self.params, self.iter)
    }

    fn using_clone<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send + 'static,
    {
        let using = UsingClone::new(using);
        UPar::new(using, self.params, self.iter)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync,
    {
        let (params, iter) = self.destruct();
        ParMap::new(params, iter, map)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync,
    {
        let (params, iter) = self.destruct();
        let x1 = move |i: Self::Item| filter(&i).then_some(i);
        ParXap::new(params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync,
    {
        let (params, iter) = self.destruct();
        let x1 = move |i: Self::Item| Vector(flat_map(i)); // TODO: inline
        ParXap::new(params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync,
    {
        let (params, iter) = self.destruct();
        ParXap::new(params, iter, filter_map)
    }

    fn take_while<While>(self, take_while: While) -> impl ParIter<R, Item = Self::Item>
    where
        While: Fn(&Self::Item) -> bool + Sync,
    {
        let (params, iter) = self.destruct();
        let x1 = move |value: Self::Item| WhilstAtom::new(value, &take_while);
        ParXap::new(params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(self) -> impl ParIterResult<R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
        Err: Send,
    {
        ParResult::new(self)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.m_collect_into::<R, _, _>(self.m())
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.m().reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item> {
        match self.params().iteration_order {
            IterationOrder::Ordered => self.m().next::<R>().1,
            IterationOrder::Arbitrary => self.m().next_any::<R>().1,
        }
    }
}
