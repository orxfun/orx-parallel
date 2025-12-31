use crate::Params;
use crate::generic_values::runner_results::{
    Fallibility, Infallible, ParallelCollect, ParallelCollectArbitrary, Stop,
};
use crate::runner::{NumSpawned, ParallelRunner};
use crate::using::executor::parallel_compute as prc;
use crate::using::using_variants::Using;
use crate::{IterationOrder, generic_values::Values};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn map_collect_into<'using, U, R, I, O, M1, P>(
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    map1: M1,
    pinned_vec: P,
) -> (NumSpawned, P)
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    O: Send,
    P: IntoConcurrentPinnedVec<O>,
{
    match (params.is_sequential(), params.iteration_order) {
        (true, _) => (
            NumSpawned::zero(),
            map_collect_into_seq(using, iter, map1, pinned_vec),
        ),
        #[cfg(test)]
        (false, IterationOrder::Arbitrary) => {
            prc::collect_arbitrary::m(using, orchestrator, params, iter, map1, pinned_vec)
        }
        (false, _) => prc::collect_ordered::m(using, orchestrator, params, iter, map1, pinned_vec),
    }
}

fn map_collect_into_seq<'using, U, I, O, M1, P>(using: U, iter: I, map1: M1, mut pinned_vec: P) -> P
where
    U: Using<'using>,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    O: Send,
    P: IntoConcurrentPinnedVec<O>,
{
    let mut u = using.into_inner();
    let iter = iter.into_seq_iter();
    for i in iter {
        pinned_vec.push(map1(&mut u, i));
    }
    pinned_vec
}

pub fn xap_collect_into<'using, U, R, I, Vo, X1, P>(
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (NumSpawned, P)
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values<Fallibility = Infallible>,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    match (params.is_sequential(), params.iteration_order) {
        (true, _) => (
            NumSpawned::zero(),
            xap_collect_into_seq(using, iter, xap1, pinned_vec),
        ),
        (false, IterationOrder::Arbitrary) => {
            let (num_threads, result) =
                prc::collect_arbitrary::x(using, orchestrator, params, iter, xap1, pinned_vec);
            let pinned_vec = match result {
                ParallelCollectArbitrary::AllOrUntilWhileCollected { pinned_vec } => pinned_vec,
            };
            (num_threads, pinned_vec)
        }
        (false, IterationOrder::Ordered) => {
            let (num_threads, result) =
                prc::collect_ordered::x(using, orchestrator, params, iter, xap1, pinned_vec);
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

fn xap_collect_into_seq<'using, U, I, Vo, X1, P>(
    using: U,
    iter: I,
    xap1: X1,
    mut pinned_vec: P,
) -> P
where
    U: Using<'using>,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let mut u = using.into_inner();
    let iter = iter.into_seq_iter();
    for i in iter {
        let vt = xap1(&mut u, i);
        let done = vt.push_to_pinned_vec(&mut pinned_vec);
        if Vo::sequential_push_to_stop(done).is_some() {
            break;
        }
    }

    pinned_vec
}

pub fn xap_try_collect_into<'using, U, R, I, Vo, X1, P>(
    using: U,
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (
    NumSpawned,
    Result<P, <Vo::Fallibility as Fallibility>::Error>,
)
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    match (params.is_sequential(), params.iteration_order) {
        (true, _) => (
            NumSpawned::zero(),
            xap_try_collect_into_seq(using, iter, xap1, pinned_vec),
        ),
        (false, IterationOrder::Arbitrary) => {
            let (nt, result) =
                prc::collect_arbitrary::x(using, orchestrator, params, iter, xap1, pinned_vec);
            (nt, result.into_result())
        }
        (false, IterationOrder::Ordered) => {
            let (nt, result) =
                prc::collect_ordered::x(using, orchestrator, params, iter, xap1, pinned_vec);
            (nt, result.into_result())
        }
    }
}

fn xap_try_collect_into_seq<'using, U, I, Vo, X1, P>(
    using: U,
    iter: I,
    xap1: X1,
    mut pinned_vec: P,
) -> Result<P, <Vo::Fallibility as Fallibility>::Error>
where
    U: Using<'using>,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let mut u = using.into_inner();
    let iter = iter.into_seq_iter();
    for i in iter {
        let vt = xap1(&mut u, i);
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
