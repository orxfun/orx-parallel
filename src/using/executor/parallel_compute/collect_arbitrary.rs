use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::ParallelCollectArbitrary;
use crate::runner::{ComputationKind, NumSpawned, ParallelRunner, SharedStateOf, ThreadRunnerOf};
use crate::using::executor::thread_compute as th;
use crate::using::using_variants::Using;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

#[cfg(test)]
pub fn m<'using, U, C, I, O, M1, P>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
    pinned_vec: P,
) -> (NumSpawned, P)
where
    U: Using<'using>,
    C: ParallelRunner,
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

pub fn x<'using, U, C, I, Vo, X1, P>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (NumSpawned, ParallelCollectArbitrary<Vo, P>)
where
    U: Using<'using>,
    C: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();

    let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    let thread_map =
        |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
            let u = using.create(nt.into_inner());
            th::collect_arbitrary::x(u, thread_runner, iter, state, &xap1, &bag).into_result()
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
