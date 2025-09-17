use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::ParallelCollectArbitrary;
use crate::orch::Orchestrator;
use crate::orch::{NumSpawned, SharedStateOf, ThreadRunnerOf};
use crate::runner::ComputationKind;
use crate::using::runner::thread_runner_compute as th;
#[cfg(test)]
use crate::using::using_variants::Using;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

#[cfg(test)]
pub fn m<U, C, I, O, M1, P>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
    pinned_vec: P,
) -> (NumSpawned, P)
where
    U: Using + Sync,
    U::Item: Send,
    C: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let mut bag: ConcurrentBag<O, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };
    let thread_work =
        |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
            let u = using.create(nt.into_inner());
            th::collect_arbitrary::m(u, thread_runner, iter, state, &map1, &bag);
        };
    let num_spawned = orchestrator.run_all(params, iter, ComputationKind::Collect, thread_work);

    let values = bag.into_inner();
    (num_spawned, values)
}
