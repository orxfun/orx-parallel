use crate::computational_variants::fallible_result::ParXapResult;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::runner::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::IntoResult;
use crate::executor::parallel_compute as prc;
use crate::using::{UParXap, UsingClone, UsingFun};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Params};
use crate::{ParIterResult, ParIterUsing};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct ParXap<I, Vo, X1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Vo + Sync,
{
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
}

impl<I, Vo, X1, R> ParXap<I, Vo, X1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Vo + Sync,
{
    pub(crate) fn new(orchestrator: R, params: Params, iter: I, xap1: X1) -> Self {
        Self {
            orchestrator,
            params,
            iter,
            xap1,
        }
    }

    pub(crate) fn destruct(self) -> (R, Params, I, X1) {
        (self.orchestrator, self.params, self.iter, self.xap1)
    }
}

unsafe impl<I, Vo, X1, R> Send for ParXap<I, Vo, X1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Vo + Sync,
{
}

unsafe impl<I, Vo, X1, R> Sync for ParXap<I, Vo, X1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Vo + Sync,
{
}

impl<I, Vo, X1, R> ParIter<R> for ParXap<I, Vo, X1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(I::Item) -> Vo + Sync,
{
    type Item = Vo::Item;

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
        let (_, params, iter, x1) = self.destruct();
        ParXap::new(orchestrator, params, iter, x1)
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
        UParXap::new(using, orchestrator, params, iter, m1)
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
        UParXap::new(using, orchestrator, params, iter, m1)
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
        let (orchestrator, params, iter, x1) = self.destruct();
        ParXapResult::new(orchestrator, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        output.x_collect_into(orchestrator, params, iter, x1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let (_, Ok(acc)) = prc::reduce::x(orchestrator, params, iter, x1, reduce);
        acc
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_num_threads, Ok(result)) = prc::next::x(orchestrator, params, iter, x1);
                result.map(|x| x.1)
            }
            IterationOrder::Arbitrary => {
                let (_num_threads, Ok(result)) = prc::next_any::x(orchestrator, params, iter, x1);
                result
            }
        }
    }
}
