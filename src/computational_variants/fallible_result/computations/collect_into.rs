use std::marker::PhantomData;

use crate::{
    IterationOrder, Params,
    computational_variants::ParXap,
    generic_values::{
        TransformableValues, Values,
        runner_results::{Fallibility, Stop},
    },
    orch::Orchestrator,
    par_iter_result::IntoResult,
    runner::parallel_runner_compute,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub struct ParResultCollectInto<R, I, T, E, Vo, X1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(I::Item) -> Vo + Sync,
    T: Send,
    E: Send,
{
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    p: PhantomData<(T, E)>,
}

impl<R, I, T, E, Vo, X1> ParResultCollectInto<R, I, T, E, Vo, X1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: IntoResult<T, E>,
    X1: Fn(I::Item) -> Vo + Sync,
    T: Send,
    E: Send,
{
    pub fn new(orchestrator: R, params: Params, iter: I, xap1: X1) -> Self {
        Self {
            orchestrator,
            params,
            iter,
            xap1,
            p: PhantomData,
        }
    }

    pub fn par_len(&self) -> Option<usize> {
        match (self.params.is_sequential(), self.iter.try_get_len()) {
            (true, _) => None, // not required to concurrent reserve when seq
            (false, x) => x,
        }
    }

    fn destruct(self) -> (R, Params, I, X1) {
        (self.orchestrator, self.params, self.iter, self.xap1)
    }

    fn seq_try_collect_into<P>(self, mut pinned_vec: P) -> Result<P, E>
    where
        P: IntoConcurrentPinnedVec<T>,
    {
        let (_, _, iter, x1) = self.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = x1(i);
            let done = vt.push_to_pinned_vec(&mut pinned_vec);
            if let Some(stop) = done.sequential_push_to_stop() {
                match stop {
                    Stop::DueToWhile => return Ok(pinned_vec),
                    Stop::DueToError { error } => return Err(error),
                }
            }
        }

        Ok(pinned_vec)
    }

    pub fn par_collect_into<P>(self, pinned_vec: P) -> (usize, Result<P, E>)
    where
        P: IntoConcurrentPinnedVec<T>,
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
}
