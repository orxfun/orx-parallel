use crate::{
    ChunkSize, DefaultRunner, IterationOrder, NumThreads, ParCollectInto, ParIterResult,
    ParallelRunner,
    par_iter_option::{ParIterOption, ResultIntoOption},
};
use std::marker::PhantomData;

pub struct ParOption<F, T, R = DefaultRunner>
where
    R: ParallelRunner,
    F: ParIterResult<R, Ok = T, Error = ()>,
{
    par: F,
    phantom: PhantomData<(T, R)>,
}

impl<F, T, R> ParOption<F, T, R>
where
    R: ParallelRunner,
    F: ParIterResult<R, Ok = T, Error = ()>,
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
    F: ParIterResult<R, Ok = T, Error = ()>,
{
    type Success = T;

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

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterOption<Q, Success = Self::Success> {
        ParOption::new(self.par.with_runner())
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterOption<R, Success = Out>
    where
        Map: Fn(Self::Success) -> Out + Sync + Clone,
        Out: Send,
    {
        ParOption::new(self.par.map(map))
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterOption<R, Success = Self::Success>
    where
        Self: Sized,
        Filter: Fn(&Self::Success) -> bool + Sync + Clone,
        Self::Success: Send,
    {
        ParOption::new(self.par.filter(filter))
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterOption<R, Success = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Success) -> IOut + Sync + Clone,
    {
        ParOption::new(self.par.flat_map(flat_map))
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterOption<R, Success = Out>
    where
        Self: Sized,
        FilterMap: Fn(Self::Success) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        ParOption::new(self.par.filter_map(filter_map))
    }

    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterOption<R, Success = Self::Success>
    where
        Self: Sized,
        Operation: Fn(&Self::Success) + Sync + Clone,
        Self::Success: Send,
    {
        ParOption::new(self.par.inspect(operation))
    }

    // collect

    fn collect_into<C>(self, output: C) -> Option<C>
    where
        C: ParCollectInto<Self::Success>,
    {
        self.par.collect_into(output).into_option()
    }

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Success>,
    {
        self.par.collect().into_option()
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Success>>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync,
    {
        self.par.reduce(reduce).into_option()
    }

    // early exit

    fn first(self) -> Option<Option<Self::Success>>
    where
        Self::Success: Send,
    {
        self.par.first().into_option()
    }
}
