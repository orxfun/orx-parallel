use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::par_iter_result::IntoResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::executor::parallel_compute as prc;
use crate::using::{ParIterResultUsing, UParXap, Using};
use crate::{IterationOrder, ParCollectInto, Params};
use core::marker::PhantomData;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub struct UParXapResult<U, I, T, E, Vo, X1, R = DefaultRunner>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    phantom: PhantomData<(T, E)>,
}

impl<U, I, T, E, Vo, X1, R> UParXapResult<U, I, T, E, Vo, X1, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: IntoResult<T, E>,
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

    fn destruct(self) -> (U, R, Params, I, X1) {
        (
            self.using,
            self.orchestrator,
            self.params,
            self.iter,
            self.xap1,
        )
    }
}

impl<U, I, T, E, Vo, X1, R> ParIterResultUsing<U, R> for UParXapResult<U, I, T, E, Vo, X1, R>
where
    U: Using,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
    type Item = T;

    type Err = E;

    type RegularItem = Vo::Item;

    type RegularParIter = UParXap<U, I, Vo, X1, R>;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    fn into_regular_par(self) -> Self::RegularParIter {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        UParXap::new(using, orchestrator, params, iter, x1)
    }

    fn from_regular_par(regular_par: Self::RegularParIter) -> Self {
        let (using, orchestrator, params, iter, x1) = regular_par.destruct();
        Self::new(using, orchestrator, params, iter, x1)
    }

    // params transformations

    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterResultUsing<U, Q, Item = Self::Item, Err = Self::Err> {
        let (using, _, params, iter, x1) = self.destruct();
        UParXapResult::new(using, orchestrator, params, iter, x1)
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Err: Send,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let x1 = |u: *mut U::Item, i: I::Item| x1(u, i).map_while_ok(|x| x.into_result());
        output.u_x_try_collect_into(using, orchestrator, params, iter, x1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let x1 = |u: *mut U::Item, i: I::Item| x1(u, i).map_while_ok(|x| x.into_result());
        prc::reduce::x(using, orchestrator, params, iter, x1, reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (using, orchestrator, params, iter, x1) = self.destruct();
        let x1 = |u: &mut U::Item, i: I::Item| x1(u, i).map_while_ok(|x| x.into_result());
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
