use crate::computational_variants::ParXap;
use crate::computational_variants::fallible_result::computations::X;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::parallel_runner_compute;
use crate::{IterationOrder, ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParXapResult<I, T, E, Vo, X1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(I::Item) -> Vo + Sync,
{
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, Vo, X1, R> ParXapResult<I, T, E, Vo, X1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(I::Item) -> Vo + Sync,
{
    pub(crate) fn new(orchestrator: R, params: Params, iter: I, xap1: X1) -> Self {
        Self {
            orchestrator,
            params,
            iter,
            xap1,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (R, Params, I, X1) {
        (self.orchestrator, self.params, self.iter, self.xap1)
    }
}

impl<I, T, E, Vo, X1, R> ParIterResult<R> for ParXapResult<I, T, E, Vo, X1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(I::Item) -> Vo + Sync,
{
    type Item = T;

    type Err = E;

    type RegularItem = Vo::Item;

    type RegularParIter = ParXap<I, Vo, X1, R>;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    fn into_regular_par(self) -> Self::RegularParIter {
        let (orchestrator, params, iter, x1) = self.destruct();
        ParXap::new(orchestrator, params, iter, x1)
    }

    fn from_regular_par(regular_par: Self::RegularParIter) -> Self {
        let (orchestrator, params, iter, x1) = regular_par.destruct();
        Self::new(orchestrator, params, iter, x1)
    }

    // params transformations

    fn with_runner<Q: Orchestrator>(
        self,
        orchestrator: Q,
    ) -> impl ParIterResult<Q, Item = Self::Item, Err = Self::Err> {
        let (_, params, iter, x1) = self.destruct();
        ParXapResult::new(orchestrator, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        output.x_try_collect_into(orchestrator, params, iter, x1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = ParXap::new(orchestrator, params, iter, x1);
        parallel_runner_compute::reduce::x(x, reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter, x1) = self.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = ParXap::new(orchestrator, params, iter, x1);
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_, result) = parallel_runner_compute::next::x(x);
                result.map(|x| x.map(|y| y.1))
            }
            IterationOrder::Arbitrary => {
                let (_, result) = parallel_runner_compute::next_any::x(x);
                result
            }
        }
    }
}
