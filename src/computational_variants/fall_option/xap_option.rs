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

    // fn map<Out, Map>(
    //     self,
    //     map: Map,
    // ) -> ParXapOption<
    //     I,
    //     Vo,
    //     impl Fn(
    //         <I as ConcurrentIter>::Item,
    //     ) -> Option<
    //         impl TransformableValues<Item = Out, Fallibility = <Vo as Values>::Fallibility>,
    //     >,
    //     R,
    // >
    // where
    //     Map: Fn(Vo::Item) -> Out + Sync + Clone,
    //     Out: Send,
    // {
    //     let (runner, params, iter, xap1) = self.par.destruct();
    //     let x1 = move |i: I::Item| xap1(i).map(|vo| vo.map(map.clone()));
    //     let a = ParXapOption::new(ParXap::new(runner, params, iter, x1));
    //     a
    // }

    // fn filter<Filter>(self, filter: Filter) -> char
    // where
    //     Self: Sized,
    //     Filter: Fn(&O) -> bool + Sync + Clone,
    // {
    //     let (runner, params, iter, xap1) = self.par.destruct();
    //     let x1 = move |i: I::Item| {
    //         xap1(i).map(|x| match filter(&x) {
    //             true => Some(x),
    //             false => None,
    //         })
    //     };
    //     let a = ParXapOption::new(ParXap::new(runner, params, iter, x1));
    //     todo!()
    // }

    // fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> char
    // where
    //     Self: Sized,
    //     IOut: IntoIterator,
    //     IOut::Item: Send,
    //     FlatMap: Fn(I::Item) -> IOut + Sync + Clone,
    // {
    //     todo!()
    // }

    // fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> char
    // where
    //     Self: Sized,
    //     FilterMap: Fn(I::Item) -> Option<Out> + Sync + Clone,
    //     Out: Send,
    // {
    //     todo!()
    // }

    // fn inspect<Operation>(self, operation: Operation) -> char
    // where
    //     Self: Sized,
    //     Operation: Fn(&I::Item) + Sync + Clone,
    //     I::Item: Send,
    // {
    //     todo!()
    // }
}
