use crate::par_iter_result::IntoResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::executor::parallel_compute as prc;
use crate::using::{UPar, ParIterResultUsing, Using};
use crate::{IterationOrder, ParCollectInto, ParIterUsing};
use core::marker::PhantomData;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub struct UParResult<U, I, T, E, R = DefaultRunner>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
{
    par: UPar<U, I, R>,
    phantom: PhantomData<(T, E)>,
}

impl<U, I, T, E, R> UParResult<U, I, T, E, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
{
    pub(crate) fn new(par: UPar<U, I, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<U, I, T, E, R> ParIterResultUsing<U, R> for UParResult<U, I, T, E, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
{
    type Item = T;

    type Err = E;

    type RegularItem = I::Item;

    type RegularParIter = UPar<U, I, R>;

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

    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterResultUsing<U, Q, Item = Self::Item, Err = Self::Err> {
        let (using, _, params, iter) = self.par.destruct();
        UParResult {
            par: UPar::new(using, orchestrator, params, iter),
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
        let (using, orchestrator, params, iter) = self.par.destruct();
        let x1 = |_: &mut U::Item, i: I::Item| i.into_result();
        output.u_x_try_collect_into(using, orchestrator, params, iter, x1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (using, orchestrator, params, iter) = self.par.destruct();
        let x1 = |_: &mut U::Item, i: I::Item| i.into_result();
        prc::reduce::x(using, orchestrator, params, iter, x1, reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (using, orchestrator, params, iter) = self.par.destruct();
        let x1 = |_: &mut U::Item, i: I::Item| i.into_result();
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_, result) = prc::next::x(using, orchestrator, params, iter, x1);
                result.map(|x| x.map(|y| y.1))
            }
            IterationOrder::Arbitrary => {
                let (_, result) = prc::next_any::x(using, orchestrator, params, iter, x1);
                result
            }
        }
    }
}
