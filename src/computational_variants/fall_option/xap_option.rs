use crate::computational_variants::ParXap;
use crate::executor::parallel_compute as prc;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params};
use core::marker::PhantomData;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub struct ParXapOption<I, Vo, X1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Option<Vo> + Sync,
{
    par: ParXap<I, Option<Vo>, X1, R>,
}

impl<I, Vo, X1, R> ParXapOption<I, Vo, X1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Option<Vo> + Sync,
{
    pub(crate) fn new(par: ParXap<I, Option<Vo>, X1, R>) -> Self {
        Self { par }
    }

    pub fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    pub fn into_regular_par(self) -> ParXap<I, Option<Vo>, X1, R> {
        self.par
    }

    pub fn from_regular_par(regular_par: ParXap<I, Option<Vo>, X1, R>) -> Self {
        Self::new(regular_par)
    }

    // params transformations

    pub fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self::new(self.par.num_threads(num_threads))
    }

    pub fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self::new(self.par.chunk_size(chunk_size))
    }

    pub fn iteration_order(self, order: IterationOrder) -> Self {
        Self::new(self.par.iteration_order(order))
    }

    pub fn with_runner<Q: ParallelRunner>(self, orchestrator: Q) -> ParXapOption<I, Vo, X1, Q> {
        let (_, params, iter, xap1) = self.par.destruct();
        ParXapOption::new(ParXap::new(orchestrator, params, iter, xap1))
    }

    // computation transformations

    pub fn map<Out, Map>(
        self,
        map: Map,
    ) -> ParXapOption<I, Vo::Map<Map, Out>, impl Fn(I::Item) -> Option<Vo::Map<Map, Out>>, R>
    where
        Map: Fn(Vo::Item) -> Out + Sync + Clone,
        Out: Send,
    {
        let (runner, params, iter, xap1) = self.par.destruct();
        let x1 = move |i: I::Item| xap1(i).map(|vo| vo.map(map.clone()));
        ParXapOption::new(ParXap::new(runner, params, iter, x1))
    }

    pub fn filter<Filter>(
        self,
        filter: Filter,
    ) -> ParXapOption<I, Vo::Filter<Filter>, impl Fn(I::Item) -> Option<Vo::Filter<Filter>>, R>
    where
        Filter: Fn(&Vo::Item) -> bool + Sync + Clone,
    {
        let (runner, params, iter, xap1) = self.par.destruct();
        let x1 = move |i: I::Item| xap1(i).map(|vo| vo.filter(filter.clone()));
        ParXapOption::new(ParXap::new(runner, params, iter, x1))
    }

    pub fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> ParXapOption<
        I,
        Vo::FlatMap<FlatMap, IOut>,
        impl Fn(I::Item) -> Option<Vo::FlatMap<FlatMap, IOut>>,
        R,
    >
    where
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Vo::Item) -> IOut + Sync + Clone,
    {
        let (runner, params, iter, xap1) = self.par.destruct();
        let x1 = move |i: I::Item| xap1(i).map(|vo| vo.flat_map(flat_map.clone()));
        ParXapOption::new(ParXap::new(runner, params, iter, x1))
    }

    pub fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> ParXapOption<
        I,
        Vo::FilterMap<FilterMap, Out>,
        impl Fn(I::Item) -> Option<Vo::FilterMap<FilterMap, Out>>,
        R,
    >
    where
        FilterMap: Fn(Vo::Item) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        let (runner, params, iter, xap1) = self.par.destruct();
        let x1 = move |i: I::Item| xap1(i).map(|vo| vo.filter_map(filter_map.clone()));
        ParXapOption::new(ParXap::new(runner, params, iter, x1))
    }

    pub fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> ParXapOption<
        I,
        Vo::Inspect<Operation>,
        impl Fn(I::Item) -> Option<Vo::Inspect<Operation>>,
        R,
    >
    where
        Operation: Fn(&Vo::Item) + Sync + Clone,
        Vo::Item: Send,
    {
        let (runner, params, iter, xap1) = self.par.destruct();
        let x1 = move |i: I::Item| xap1(i).map(|vo| vo.inspect(operation.clone()));
        ParXapOption::new(ParXap::new(runner, params, iter, x1))
    }
}
