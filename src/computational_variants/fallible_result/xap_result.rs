use crate::computational_variants::ParXap;
use crate::computations::X;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParXapResult<I, T, E, Vo, M1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    par: ParXap<I, Vo, M1, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, Vo, M1, R> ParXapResult<I, T, E, Vo, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub(crate) fn new(par: ParXap<I, Vo, M1, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, Vo, M1, R> ParIterResult<R> for ParXapResult<I, T, E, Vo, M1, R>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    type Item = T;

    type Err = E;

    type RegularItem = Vo::Item;

    type RegularParIter = ParXap<I, Vo, M1, R>;

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
        let (orchestrator, params, iter, m1) = self.par.destruct();
        ParXapResult {
            par: ParXap::new(orchestrator, params, iter, m1),
            phantom: PhantomData,
        }
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Err: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter, x1) = self.par.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (orchestrator, params, iter, x1) = self.par.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (orchestrator, params, iter, x1) = self.par.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = X::new(params, iter, x1);
        match params.iteration_order {
            IterationOrder::Ordered => x.try_next::<R>().1,
            IterationOrder::Arbitrary => x.try_next_any::<R>().1,
        }
    }
}
