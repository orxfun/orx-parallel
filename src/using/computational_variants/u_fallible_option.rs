use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto,
    par_iter_option::ResultIntoOption,
    runner::{DefaultRunner, ParallelRunner},
    using::{ParIterOptionUsing, ParIterResultUsing, Using},
};
use core::marker::PhantomData;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub struct UParOption<U, F, T, R = DefaultRunner>
where
    R: ParallelRunner,
    F: ParIterResultUsing<U, R, Item = T, Err = ()>,
    U: Using,
{
    par: F,
    phantom: PhantomData<(U, T, R)>,
}

impl<U, F, T, R> UParOption<U, F, T, R>
where
    R: ParallelRunner,
    F: ParIterResultUsing<U, R, Item = T, Err = ()>,
    U: Using,
{
    pub(crate) fn new(par: F) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<U, F, T, R> ParIterOptionUsing<U, R> for UParOption<U, F, T, R>
where
    R: ParallelRunner,
    F: ParIterResultUsing<U, R, Item = T, Err = ()>,
    U: Using,
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
    ) -> impl ParIterOptionUsing<U, Q, Item = Self::Item> {
        UParOption::<U, _, _, _>::new(self.par.with_runner(orchestrator))
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterOptionUsing<U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
        Out: Send,
    {
        UParOption::<U, _, _, _>::new(self.par.map(map))
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterOptionUsing<U, R, Item = Self::Item>
    where
        Self: Sized,
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
        Self::Item: Send,
    {
        UParOption::<U, _, _, _>::new(self.par.filter(filter))
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterOptionUsing<U, R, Item = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        UParOption::<U, _, _, _>::new(self.par.flat_map(flat_map))
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterOptionUsing<U, R, Item = Out>
    where
        Self: Sized,
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        UParOption::<U, _, _, _>::new(self.par.filter_map(filter_map))
    }

    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterOptionUsing<U, R, Item = Self::Item>
    where
        Self: Sized,
        Operation: Fn(&mut U::Item, &Self::Item) + Sync + Clone,
        Self::Item: Send,
    {
        UParOption::<U, _, _, _>::new(self.par.inspect(operation))
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
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
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
