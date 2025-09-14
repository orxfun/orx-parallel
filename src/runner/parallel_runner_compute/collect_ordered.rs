use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, ParallelCollect};
use crate::orch::{NumSpawned, Orchestrator, SharedStateOf, ThreadRunnerOf};
use crate::runner::{ComputationKind, thread_runner_compute as thread};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn m<C, I, O, M1, P>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
    pinned_vec: P,
) -> (NumSpawned, P)
where
    C: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let offset = pinned_vec.len();
    let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    let thread_do = |iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        thread::collect_ordered::m(thread_runner, iter, state, &map1, &o_bag, offset);
    };
    let num_spawned = orchestrator.run_all(params, iter, ComputationKind::Collect, thread_do);

    let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
    (num_spawned, values)
}

// x

pub fn x<C, I, Vo, X1, P>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (NumSpawned, ParallelCollect<Vo, P>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    <Vo::Fallibility as Fallibility>::Error: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let thread_map = |iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        thread::collect_ordered::x(thread_runner, iter, state, &xap1).into_result()
    };
    let (num_spawned, results) =
        orchestrator.map_all(params, iter, ComputationKind::Collect, thread_map);

    let result = match results {
        Err(error) => ParallelCollect::StoppedByError { error },
        Ok(results) => ParallelCollect::reduce(results, pinned_vec),
    };
    (num_spawned, result)
}
