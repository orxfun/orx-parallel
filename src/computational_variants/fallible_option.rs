use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIterResult,
    runner::{DefaultOrchestrator, ParallelRunner},
    par_iter_option::{ParIterOption, ResultIntoOption},
};
use core::marker::PhantomData;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub struct ParOption<F, T, R = DefaultOrchestrator>
where
    R: ParallelRunner,
    F: ParIterResult<R, Item = T, Err = ()>,
{
    par: F,
    phantom: PhantomData<(T, R)>,
}

impl<F, T, R> ParOption<F, T, R>
where
    R: ParallelRunner,
    F: ParIterResult<R, Item = T, Err = ()>,
{
    pub(crate) fn new(par: F) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<F, T, R> ParIterOption<R> for ParOption<F, T, R>
where
    R: ParallelRunner,
    F: ParIterResult<R, Item = T, Err = ()>,
{
    type Item = T;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self::new(self.par.num_threads(num_threads))
    }

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self::new(self.par.chunk_size(chunk_size))
    }

    fn iteration_order(self, order: IterationOrder) -> Self {
        Self::new(self.par.iteration_order(order))
    }

    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterOption<Q, Item = Self::Item> {
        ParOption::new(self.par.with_runner(orchestrator))
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterOption<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync + Clone,
        Out: Send,
    {
        ParOption::new(self.par.map(map))
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterOption<R, Item = Self::Item>
    where
        Self: Sized,
        Filter: Fn(&Self::Item) -> bool + Sync + Clone,
        Self::Item: Send,
    {
        ParOption::new(self.par.filter(filter))
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIterOption<R, Item = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone,
    {
        ParOption::new(self.par.flat_map(flat_map))
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIterOption<R, Item = Out>
    where
        Self: Sized,
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        ParOption::new(self.par.filter_map(filter_map))
    }

    fn inspect<Operation>(self, operation: Operation) -> impl ParIterOption<R, Item = Self::Item>
    where
        Self: Sized,
        Operation: Fn(&Self::Item) + Sync + Clone,
        Self::Item: Send,
    {
        ParOption::new(self.par.inspect(operation))
    }

    // collect

    fn collect_into<C>(self, output: C) -> Option<C>
    where
        Self::Item: Send,
        C: ParCollectInto<Self::Item>,
    {
        self.par.collect_into(output).into_option()
    }

    fn collect<C>(self) -> Option<C>
    where
        Self::Item: Send,
        C: ParCollectInto<Self::Item>,
    {
        self.par.collect().into_option()
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.par.reduce(reduce).into_option()
    }

    // early exit

    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
    {
        self.par.first().into_option()
    }
}
