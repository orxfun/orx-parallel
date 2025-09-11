use crate::Params;
use crate::generic_values::runner_results::{
    Fallibility, Infallible, ParallelCollect, ParallelCollectArbitrary, Stop,
};
use crate::orch::Orchestrator;
use crate::runner::parallel_runner_compute;
use crate::{IterationOrder, generic_values::Values};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn collect_into<R, I, Vo, X1, P>(
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (usize, P)
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values<Fallibility = Infallible>,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    match (params.is_sequential(), params.iteration_order) {
        (true, _) => (0, sequential(iter, xap1, pinned_vec)),
        (false, IterationOrder::Arbitrary) => {
            let (num_threads, result) = parallel_runner_compute::collect_arbitrary::x(
                orchestrator,
                params,
                iter,
                xap1,
                pinned_vec,
            );
            let pinned_vec = match result {
                ParallelCollectArbitrary::AllCollected { pinned_vec } => pinned_vec,
                ParallelCollectArbitrary::StoppedByWhileCondition { pinned_vec } => pinned_vec,
            };
            (num_threads, pinned_vec)
        }
        (false, IterationOrder::Ordered) => {
            let (num_threads, result) = parallel_runner_compute::collect_ordered::x(
                orchestrator,
                params,
                iter,
                xap1,
                pinned_vec,
            );
            let pinned_vec = match result {
                ParallelCollect::AllCollected { pinned_vec } => pinned_vec,
                ParallelCollect::StoppedByWhileCondition {
                    pinned_vec,
                    stopped_idx: _,
                } => pinned_vec,
            };
            (num_threads, pinned_vec)
        }
    }
}

fn sequential<I, Vo, X1, P>(iter: I, xap1: X1, mut pinned_vec: P) -> P
where
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let iter = iter.into_seq_iter();
    for i in iter {
        let vt = xap1(i);
        let done = vt.push_to_pinned_vec(&mut pinned_vec);
        if Vo::sequential_push_to_stop(done).is_some() {
            break;
        }
    }

    pinned_vec
}

pub fn try_collect_into<R, I, Vo, X1, P>(
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (usize, Result<P, <Vo::Fallibility as Fallibility>::Error>)
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    match (params.is_sequential(), params.iteration_order) {
        (true, _) => (0, try_sequential(iter, xap1, pinned_vec)),
        (false, IterationOrder::Arbitrary) => {
            let (nt, result) = parallel_runner_compute::collect_arbitrary::x(
                orchestrator,
                params,
                iter,
                xap1,
                pinned_vec,
            );
            (nt, result.into_result())
        }
        (false, IterationOrder::Ordered) => {
            let (nt, result) = parallel_runner_compute::collect_ordered::x(
                orchestrator,
                params,
                iter,
                xap1,
                pinned_vec,
            );
            (nt, result.into_result())
        }
    }
}

fn try_sequential<I, Vo, X1, P>(
    iter: I,
    xap1: X1,
    mut pinned_vec: P,
) -> Result<P, <Vo::Fallibility as Fallibility>::Error>
where
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let iter = iter.into_seq_iter();
    for i in iter {
        let vt = xap1(i);
        let done = vt.push_to_pinned_vec(&mut pinned_vec);
        if let Some(stop) = Vo::sequential_push_to_stop(done) {
            match stop {
                Stop::DueToWhile => return Ok(pinned_vec),
                Stop::DueToError { error } => return Err(error),
            }
        }
    }

    Ok(pinned_vec)
}
