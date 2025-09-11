use crate::computational_variants::ParMap;
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::parallel_runner_compute;
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub struct ParMapResult<I, T, E, O, M1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(I::Item) -> O + Sync,
{
    par: ParMap<I, O, M1, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, O, M1, R> ParMapResult<I, T, E, O, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(I::Item) -> O + Sync,
{
    pub(crate) fn new(par: ParMap<I, O, M1, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, O, M1, R> ParIterResult<R> for ParMapResult<I, T, E, O, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(I::Item) -> O + Sync,
{
    type Item = T;

    type Err = E;

    type RegularItem = O;

    type RegularParIter = ParMap<I, O, M1, R>;

    fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    fn into_regular_par(self) -> Self::RegularParIter {
        self.par
    }

    fn from_regular_par(regular_par: Self::RegularParIter) -> Self {
        Self {
            par: regular_par,
            phantom: PhantomData,
        }
    }

    // params transformations

    fn with_runner<Q: Orchestrator>(
        self,
        orchestrator: Q,
    ) -> impl ParIterResult<Q, Item = Self::Item, Err = Self::Err> {
        let (_, params, iter, m1) = self.par.destruct();
        ParMapResult {
            par: ParMap::new(orchestrator, params, iter, m1),
            phantom: PhantomData,
        }
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter, m1) = self.par.destruct();
        let x1 = |i: I::Item| m1(i).into_result();
        output.x_try_collect_into(orchestrator, params, iter, x1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter, m1) = self.par.destruct();
        let x1 = |i: I::Item| m1(i).into_result();
        parallel_runner_compute::reduce::x(orchestrator, params, iter, x1, reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter, m1) = self.par.destruct();
        let x1 = |i: I::Item| m1(i).into_result();
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_, result) = parallel_runner_compute::next::x(orchestrator, params, iter, x1);
                result.map(|x| x.map(|y| y.1))
            }
            IterationOrder::Arbitrary => {
                let (_, result) =
                    parallel_runner_compute::next_any::x(orchestrator, params, iter, x1);
                result
            }
        }
    }
}
