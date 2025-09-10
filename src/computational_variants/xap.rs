use crate::ParIterResult;
use crate::computational_variants::fallible_result::ParXapResult;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::IntoResult;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    computations::X,
    using::{UsingClone, UsingFun, computational_variants::UParXap},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct ParXap<I, Vo, M1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    orchestrator: R,
    x: X<I, Vo, M1>,
    phantom: PhantomData<R>,
}

impl<I, Vo, M1, R> ParXap<I, Vo, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub(crate) fn new(orchestrator: R, params: Params, iter: I, x1: M1) -> Self {
        Self {
            orchestrator,
            x: X::new(params, iter, x1),
            phantom: PhantomData,
        }
    }

    pub(crate) fn destruct(self) -> (R, Params, I, M1) {
        let (params, iter, x1) = self.x.destruct();
        (self.orchestrator, params, iter, x1)
    }
}

unsafe impl<I, Vo, M1, R> Send for ParXap<I, Vo, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(I::Item) -> Vo + Sync,
{
}

unsafe impl<I, Vo, M1, R> Sync for ParXap<I, Vo, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(I::Item) -> Vo + Sync,
{
}

impl<I, Vo, M1, R> ParIter<R> for ParXap<I, Vo, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.x.iter()
    }

    fn params(&self) -> Params {
        self.x.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.x.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.x.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.x.iteration_order(collect);
        self
    }

    fn with_runner<Q: Orchestrator>(self, orchestrator: Q) -> impl ParIter<Q, Item = Self::Item> {
        let (_, params, iter, x1) = self.destruct();
        ParXap::new(orchestrator, params, iter, x1)
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
        let (orchestrator, params, iter, x1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| x1(t);
        UParXap::new(using, params, iter, m1)
    }

    fn using_clone<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send + 'static,
    {
        let using = UsingClone::new(using);
        let (orchestrator, params, iter, x1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| x1(t);
        UParXap::new(using, params, iter, m1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync + Clone,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.map(map.clone())
        };

        ParXap::new(orchestrator, params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync + Clone,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let values = x1(i);
            values.filter(filter.clone())
        };
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.flat_map(flat_map.clone())
        };
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.filter_map(filter_map.clone())
        };
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn take_while<While>(self, take_while: While) -> impl ParIter<R, Item = Self::Item>
    where
        While: Fn(&Self::Item) -> bool + Sync + Clone,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.whilst(take_while.clone())
        };
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(self) -> impl ParIterResult<R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
    {
        ParXapResult::new(self)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.x_collect_into::<R::Runner, _, _, _>(self.x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.x.reduce::<R::Runner, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        match self.params().iteration_order {
            IterationOrder::Ordered => self.x.next::<R::Runner>().1,
            IterationOrder::Arbitrary => self.x.next_any::<R::Runner>().1,
        }
    }
}
