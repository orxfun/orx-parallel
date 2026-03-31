use core::marker::PhantomData;

use crate::par_iter_result::IntoResult;
use crate::using::ParIterResultUsing;
use crate::using::computational_variants::u_fallible_result::UParXapResult;
use crate::using::executor::parallel_compute as prc;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIterUsing, Params,
    generic_values::{TransformableValues, runner_results::Infallible},
    runner::{DefaultRunner, ParallelRunner},
    using::using_variants::Using,
};
use orx_concurrent_iter::ConcurrentIter;
// use crate::runner::parallel_runner_compute as prc;

pub struct UParXap<'using, U, I, Vo, X1, R = DefaultRunner>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    phantom: PhantomData<&'using ()>,
}

impl<'using, U, I, Vo, X1, R> UParXap<'using, U, I, Vo, X1, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
    pub(crate) fn new(using: U, orchestrator: R, params: Params, iter: I, xap1: X1) -> Self {
        Self {
            using,
            orchestrator,
            params,
            iter,
            xap1,
            phantom: PhantomData,
        }
    }

    pub(crate) fn destruct(self) -> (U, R, Params, I, X1) {
        (
            self.using,
            self.orchestrator,
            self.params,
            self.iter,
            self.xap1,
        )
    }
}

unsafe impl<'using, U, I, Vo, X1, R> Send for UParXap<'using, U, I, Vo, X1, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
}

unsafe impl<'using, U, I, Vo, X1, R> Sync for UParXap<'using, U, I, Vo, X1, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
}

impl<'using, U, I, Vo, X1, R> ParIterUsing<'using, U, R> for UParXap<'using, U, I, Vo, X1, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        &self.iter
    }

    fn params(&self) -> Params {
        self.params
    }

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

    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterUsing<'using, U, Q, Item = Self::Item> {
        let (using, _, params, iter, x1) = self.destruct();
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<'using, U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();

        let map = move |u: *mut U::Item, i: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            map(u, i)
        };

        let x1 = move |u: *mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.u_map(u, map.clone())
        };

        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<'using, U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();

        let filter = move |u: *mut U::Item, i: &Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            filter(u, i)
        };

        let x1 = move |u: *mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.u_filter(u, filter.clone())
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<'using, U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();

        let flat_map = move |u: *mut U::Item, i: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            flat_map(u, i)
        };

        let x1 = move |u: *mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.u_flat_map(u, flat_map.clone())
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<'using, U, R, Item = Out>
    where
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();

        let filter_map = move |u: *mut U::Item, i: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            filter_map(u, i)
        };

        let x1 = move |u: *mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            vo.u_filter_map(u, filter_map.clone())
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn into_fallible_result<Out, Err>(
        self,
    ) -> impl ParIterResultUsing<'using, U, R, Item = Out, Err = Err>
    where
        Self::Item: IntoResult<Out, Err>,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        UParXapResult::new(using, orchestrator, params, iter, x1)
    }

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        output.u_x_collect_into(using, orchestrator, params, iter, x1)
    }

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let reduce = move |u: *mut U::Item, a: Self::Item, b: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            reduce(u, a, b)
        };

        let (_, Ok(acc)) = prc::reduce::x(using, orchestrator, params, iter, x1, reduce);
        acc
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_num_threads, Ok(result)) =
                    prc::next::x(using, orchestrator, params, iter, x1);
                result.map(|x| x.1)
            }
            IterationOrder::Arbitrary => {
                let (_num_threads, Ok(result)) =
                    prc::next_any::x(using, orchestrator, params, iter, x1);
                result
            }
        }
    }
}
