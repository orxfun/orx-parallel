use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{ParallelCollectArbitrary, ThreadCollectArbitrary};
use crate::orch::NumSpawned;
use crate::orch::{Orchestrator, ParHandle, ParScope, ParThreadPool};
use crate::runner::ParallelRunner;
use crate::runner::{ComputationKind, thread_runner_compute as thread};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

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
    use crate::orch::{SharedStateOf, ThreadRunnerOf};

    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let mut bag: ConcurrentBag<O, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    let thread_work = |iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        thread::collect_arbitrary::m(thread_runner, iter, state, &map1, &bag);
    };
    let num_spawned = orchestrator.run_all(params, iter, ComputationKind::Collect, thread_work);

    let values = bag.into_inner();
    (num_spawned, values)
}

// x

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

    let runner = C::new_runner(ComputationKind::Collect, params, iter.try_get_len());

    let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    // compute

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = NumSpawned::zero();
    let result: ThreadCollectArbitrary<Vo::Fallibility> =
        orchestrator.thread_pool().scope_zzz(|s| {
            let mut handles = vec![];

            while runner.do_spawn_new(num_spawned, shared_state, &iter) {
                num_spawned.increment();
                handles.push(s.spawn(|| {
                    thread::collect_arbitrary::x(
                        runner.new_thread_runner(shared_state),
                        &iter,
                        shared_state,
                        &xap1,
                        &bag,
                    )
                }));
            }

            let mut early_exit_result = None;
            while !handles.is_empty() {
                let mut finished_idx = None;
                for (h, handle) in handles.iter().enumerate() {
                    if handle.is_finished() {
                        finished_idx = Some(h);
                        break;
                    }
                }

                if let Some(h) = finished_idx {
                    let handle = handles.remove(h);
                    let result = handle.join().expect("failed to join the thread");
                    match &result {
                        ThreadCollectArbitrary::AllCollected => {}
                        ThreadCollectArbitrary::StoppedByError { error: _ } => {
                            early_exit_result = Some(result);
                            break;
                        }
                        ThreadCollectArbitrary::StoppedByWhileCondition => {
                            early_exit_result = Some(result);
                        }
                    }
                }
            }

            early_exit_result.unwrap_or(ThreadCollectArbitrary::AllCollected)
        });

    (
        num_spawned,
        match result {
            ThreadCollectArbitrary::AllCollected => ParallelCollectArbitrary::AllCollected {
                pinned_vec: bag.into_inner(),
            },
            ThreadCollectArbitrary::StoppedByWhileCondition => {
                ParallelCollectArbitrary::StoppedByWhileCondition {
                    pinned_vec: bag.into_inner(),
                }
            }
            ThreadCollectArbitrary::StoppedByError { error } => {
                ParallelCollectArbitrary::StoppedByError { error }
            }
        },
    )
}
