use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, Params,
    generic_values::{TransformableValues, runner_results::Infallible},
    orch::{DefaultOrchestrator, Orchestrator},
    using::{Using, computations::UX, u_par_iter::ParIterUsing},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct UParXap<U, I, Vo, M1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    ux: UX<U, I, Vo, M1>,
    phantom: PhantomData<R>,
}

impl<U, I, Vo, M1, R> UParXap<U, I, Vo, M1, R>
where
    R: Orchestrator,
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    pub(crate) fn new(using: U, params: Params, iter: I, x1: M1) -> Self {
        Self {
            ux: UX::new(using, params, iter, x1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (U, Params, I, M1) {
        self.ux.destruct()
    }
}

unsafe impl<U, I, Vo, M1, R> Send for UParXap<U, I, Vo, M1, R>
where
    R: Orchestrator,
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
}

unsafe impl<U, I, Vo, M1, R> Sync for UParXap<U, I, Vo, M1, R>
where
    R: Orchestrator,
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
}

impl<U, I, Vo, M1, R> ParIterUsing<U, R> for UParXap<U, I, Vo, M1, R>
where
    R: Orchestrator,
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.ux.iter()
    }

    fn params(&self) -> Params {
        self.ux.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.ux.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.ux.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.ux.iteration_order(collect);
        self
    }

    fn with_runner<Q: Orchestrator>(self) -> impl ParIterUsing<U, Q, Item = Self::Item> {
        let (using, params, iter, map1) = self.destruct();
        UParXap::new(using, params, iter, map1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();

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

        UParXap::new(using, params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();
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
        UParXap::new(using, params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();
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
        UParXap::new(using, params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (using, params, iter, x1) = self.destruct();
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
        UParXap::new(using, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.u_x_collect_into::<R::Runner, _, _, _, _>(self.ux)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.ux.reduce::<R::Runner, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        self.ux.next::<R::Runner>().1
    }
}
