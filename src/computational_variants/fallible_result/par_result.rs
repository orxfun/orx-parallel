use crate::computational_variants::Par;
use crate::computational_variants::fallible_result::computations::X;
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParResult<I, T, E, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
{
    par: Par<I, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, R> ParResult<I, T, E, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
{
    pub(crate) fn new(par: Par<I, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, R> ParIterResult<R> for ParResult<I, T, E, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
{
    type Item = T;

    type Err = E;

    type RegularItem = I::Item;

    type RegularParIter = Par<I, R>;

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
        let (_, params, iter) = self.par.destruct();
        ParResult {
            par: Par::new(orchestrator, params, iter),
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
        let (orchestrator, params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(orchestrator, params, iter, x1);
        output.x_try_collect_into(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(orchestrator, params, iter, x1);
        x.try_reduce(reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(orchestrator, params, iter, x1);
        match params.iteration_order {
            IterationOrder::Ordered => x.try_next().1,
            IterationOrder::Arbitrary => x.try_next_any().1,
        }
    }
}
