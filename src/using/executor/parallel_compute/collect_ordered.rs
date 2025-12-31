use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, ParallelCollect};
use crate::runner::{ComputationKind, NumSpawned, ParallelRunner, SharedStateOf, ThreadRunnerOf};
use crate::using::executor::thread_compute as th;
use crate::using::using_variants::Using;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

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
    let offset = pinned_vec.len();
    let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    let thread_do =
        |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
            let u = using.create(nt.into_inner());
            th::collect_ordered::m(u, thread_runner, iter, state, &map1, &o_bag, offset);
        };
    let num_spawned = orchestrator.run_all(params, iter, ComputationKind::Collect, thread_do);

    let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
    (num_spawned, values)
}

pub fn x<'using, U, C, I, Vo, X1, P>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
    pinned_vec: P,
) -> (NumSpawned, ParallelCollect<Vo, P>)
where
    U: Using<'using>,
    C: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    <Vo::Fallibility as Fallibility>::Error: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let thread_map =
        |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
            let u = using.create(nt.into_inner());
            th::collect_ordered::x(u, thread_runner, iter, state, &xap1).into_result()
        };
    let (num_spawned, result) = orchestrator.map_all::<Vo::Fallibility, _, _, _>(
        params,
        iter,
        ComputationKind::Collect,
        thread_map,
    );

    let result = match result {
        Err(error) => ParallelCollect::StoppedByError { error },
        Ok(results) => ParallelCollect::reduce(results, pinned_vec),
    };
    (num_spawned, result)
}
