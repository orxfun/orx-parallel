use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::ParallelCollectArbitrary;
use crate::orch::Orchestrator;
use crate::orch::{NumSpawned, SharedStateOf, ThreadRunnerOf};
use crate::runner::{ComputationKind, thread_compute as th};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

#[cfg(test)]
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
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let mut bag: ConcurrentBag<O, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    let thread_work = |_, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        th::collect_arbitrary::m(thread_runner, iter, state, &map1, &bag);
    };
    let num_spawned = orchestrator.run_all(params, iter, ComputationKind::Collect, thread_work);

    let values = bag.into_inner();
    (num_spawned, values)
}

pub fn x<C, I, Vo, X1, P>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (NumSpawned, ParallelCollectArbitrary<Vo, P>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();

    let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        th::collect_arbitrary::x(thread_runner, iter, state, &xap1, &bag).into_result()
    };
    let (num_spawned, result) = orchestrator.map_all::<Vo::Fallibility, _, _, _>(
        params,
        iter,
        ComputationKind::Collect,
        thread_map,
    );

    let result = match result {
        Err(error) => ParallelCollectArbitrary::StoppedByError { error },
        Ok(_) => ParallelCollectArbitrary::AllOrUntilWhileCollected {
            pinned_vec: bag.into_inner(),
        },
    };

    (num_spawned, result)
}
