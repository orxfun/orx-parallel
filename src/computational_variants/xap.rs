use crate::ParIterResult;
use crate::computational_variants::fallible_result::ParXapResult;
use crate::generic_values::runner_results::{
    Infallible, ParallelCollect, ParallelCollectArbitrary,
};
use crate::generic_values::{TransformableValues, Values};
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::IntoResult;
use crate::runner::parallel_runner_compute;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    using::{UsingClone, UsingFun, computational_variants::UParXap},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct ParXap<I, Vo, X1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
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
    Vo: Values,
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

    pub(crate) fn par_len(&self) -> Option<usize> {
        match (self.params.is_sequential(), self.iter.try_get_len()) {
            (true, _) => None, // not required to concurrent reserve when seq
            (false, x) => x,
        }
    }

    pub(crate) fn par_collect_into<P>(self, pinned_vec: P) -> (usize, P)
    where
        P: IntoConcurrentPinnedVec<Vo::Item>,
        Vo: TransformableValues<Fallibility = Infallible>,
        Vo::Item: Send,
    {
        match (self.params.is_sequential(), self.params.iteration_order) {
            (true, _) => (0, self.seq_collect_into(pinned_vec)),
            (false, IterationOrder::Arbitrary) => {
                let (num_threads, result) =
                    parallel_runner_compute::collect_arbitrary::x(self, pinned_vec);
                let pinned_vec = match result {
                    ParallelCollectArbitrary::AllCollected { pinned_vec } => pinned_vec,
                    ParallelCollectArbitrary::StoppedByWhileCondition { pinned_vec } => pinned_vec,
                };
                (num_threads, pinned_vec)
            }
            (false, IterationOrder::Ordered) => {
                let (num_threads, result) =
                    parallel_runner_compute::collect_ordered::x(self, pinned_vec);
                let pinned_vec = match result {
                    ParallelCollect::AllCollected { pinned_vec } => pinned_vec,
                    ParallelCollect::StoppedByWhileCondition {
                        pinned_vec,
                        stopped_idx: _,
                    } => pinned_vec,
                };
                (num_threads, pinned_vec)
            }
        }
    }

    fn seq_collect_into<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (_, _, iter, xap1) = self.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(i);
            let done = vt.push_to_pinned_vec(&mut pinned_vec);
            if Vo::sequential_push_to_stop(done).is_some() {
                break;
            }
        }

        pinned_vec
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
        output.x_collect_into(self)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (_, Ok(acc)) = parallel_runner_compute::reduce::x(self, reduce);
        acc
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => {
                let (_num_threads, Ok(result)) = parallel_runner_compute::next::x(self);
                result.map(|x| x.1)
            }
            IterationOrder::Arbitrary => {
                let (_num_threads, Ok(result)) = parallel_runner_compute::next_any::x(self);
                result
            }
        }
    }
}
