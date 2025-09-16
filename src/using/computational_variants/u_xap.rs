use crate::{
    ChunkSize, IterationOrder, NumThreads, ParIterUsing, Params,
    generic_values::{TransformableValues, runner_results::Infallible},
    orch::{DefaultOrchestrator, Orchestrator},
    using::using_variants::Using,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct UParXap<U, I, Vo, X1, R = DefaultOrchestrator>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
}

impl<U, I, Vo, X1, R> UParXap<U, I, Vo, X1, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    pub(crate) fn new(using: U, orchestrator: R, params: Params, iter: I, xap1: X1) -> Self {
        Self {
            using,
            orchestrator,
            params,
            iter,
            xap1,
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

unsafe impl<U, I, Vo, X1, R> Send for UParXap<U, I, Vo, X1, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
}

unsafe impl<U, I, Vo, X1, R> Sync for UParXap<U, I, Vo, X1, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
}

impl<U, I, Vo, X1, R> ParIterUsing<U, R> for UParXap<U, I, Vo, X1, R>
where
    U: Using,
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
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

    fn with_runner<Q: Orchestrator>(
        self,
        orchestrator: Q,
    ) -> impl ParIterUsing<U, Q, Item = Self::Item> {
        let (using, _, params, iter, x1) = self.destruct();
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Map: Fn(&mut <U as Using>::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();

        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            // SAFETY: all threads are guaranteed to have its own Using::Item value that is not shared with other threads.
            // This guarantees that there will be no race conditions.
            // TODO: the reason to have this unsafe block is the complication in lifetimes, which must be possible to fix; however with a large refactoring.
            let u = unsafe {
                &mut *{
                    let p: *mut U::Item = u;
                    p
                }
            };
            vo.u_map(u, map.clone())
        };

        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut <U as Using>::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            // SAFETY: all threads are guaranteed to have its own Using::Item value that is not shared with other threads.
            // This guarantees that there will be no race conditions.
            // TODO: the reason to have this unsafe block is the complication in lifetimes, which must be possible to fix; however with a large refactoring.
            let u = unsafe {
                &mut *{
                    let p: *mut U::Item = u;
                    p
                }
            };
            vo.u_filter(u, filter.clone())
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut <U as Using>::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            // SAFETY: all threads are guaranteed to have its own Using::Item value that is not shared with other threads.
            // This guarantees that there will be no race conditions.
            // TODO: the reason to have this unsafe block is the complication in lifetimes, which must be possible to fix; however with a large refactoring.
            let u = unsafe {
                &mut *{
                    let p: *mut U::Item = u;
                    p
                }
            };
            vo.u_flat_map(u, flat_map.clone())
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        FilterMap: Fn(&mut <U as Using>::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let x1 = move |u: &mut U::Item, i: I::Item| {
            let vo = x1(u, i);
            // SAFETY: all threads are guaranteed to have its own Using::Item value that is not shared with other threads.
            // This guarantees that there will be no race conditions.
            // TODO: the reason to have this unsafe block is the complication in lifetimes, which must be possible to fix; however with a large refactoring.
            let u = unsafe {
                &mut *{
                    let p: *mut U::Item = u;
                    p
                }
            };
            vo.u_filter_map(u, filter_map.clone())
        };
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn collect_into<C>(self, output: C) -> C
    where
        C: crate::ParCollectInto<Self::Item>,
    {
        todo!()
    }

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(&mut <U as Using>::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        todo!()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        todo!()
    }
}
