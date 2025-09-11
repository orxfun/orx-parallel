use crate::computational_variants::ParXap;
use crate::computations::X;
use crate::generic_values::runner_results::{Fallibility, Fallible, Infallible, Stop};
use crate::generic_values::{TransformableValues, Values};
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::parallel_runner_compute;
use crate::{IterationOrder, ParCollectInto, ParIter, Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct ParXapResult<I, T, E, Vo, X1, R = DefaultOrchestrator>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
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
    Vo: TransformableValues<Fallibility = Infallible>,
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

    pub(crate) fn par_collect_into<P>(self, pinned_vec: P) -> (usize, Result<P, E>)
    where
        P: IntoConcurrentPinnedVec<T>,
        Vo::Item: Send,
        E: Send,
        T: Send,
    {
        match (self.params.is_sequential(), self.params.iteration_order) {
            (true, _) => (0, self.seq_try_collect_into(pinned_vec)),

            (false, IterationOrder::Arbitrary) => {
                let (orchestrator, params, iter, x1) = self.destruct();
                let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
                let x = ParXap::new(orchestrator, params, iter, x1);
                let (nt, result) = parallel_runner_compute::collect_arbitrary::x(x, pinned_vec);
                (nt, result.into_result())
            }

            (false, IterationOrder::Ordered) => {
                let (orchestrator, params, iter, x1) = self.destruct();
                let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
                let x = ParXap::new(orchestrator, params, iter, x1);
                let (nt, result) = parallel_runner_compute::collect_ordered::x(x, pinned_vec);
                (nt, result.into_result())
            }
        }
    }

    fn seq_try_collect_into<P>(self, mut pinned_vec: P) -> Result<P, E>
    where
        P: IntoConcurrentPinnedVec<T>,
        E: Send,
    {
        let (_, _, iter, x1) = self.destruct();
        let iter = iter.into_seq_iter();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());

        for i in iter {
            let vt = x1(i);
            let done = vt.push_to_pinned_vec(&mut pinned_vec);
            if let Some(stop) = Fallible::<E>::sequential_push_to_stop(done) {
                match stop {
                    Stop::DueToWhile => return Ok(pinned_vec),
                    Stop::DueToError { error } => return Err(error),
                }
            }
        }

        Ok(pinned_vec)
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
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R::Runner, _, _, _>(x)
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
