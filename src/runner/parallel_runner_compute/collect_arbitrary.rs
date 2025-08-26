#[cfg(test)]
use crate::computations::M;
use crate::runner::thread_runner_compute as thread;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{ParallelCollectArbitrary, ThreadCollectArbitrary};
use crate::{computations::X, runner::ParallelRunnerCompute};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

#[cfg(test)]
pub fn m<C, I, O, M1, P>(runner: C, m: M<I, O, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let (_, iter, map1) = m.destruct();

    let mut bag: ConcurrentBag<O, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    // compute

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            s.spawn(|| {
                thread::collect_arbitrary::m(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &map1,
                    &bag,
                );
            });
        }
    });
    let values = bag.into_inner();
    (num_spawned, values)
}

// x

pub fn x<C, I, Vo, M1, P>(
    runner: C,
    x: X<I, Vo, M1>,
    pinned_vec: P,
) -> (usize, ParallelCollectArbitrary<Vo, P>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let (_, iter, xap1) = x.destruct();

    let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
    match iter.try_get_len() {
        Some(iter_len) => bag.reserve_maximum_capacity(offset + iter_len),
        None => bag.reserve_maximum_capacity(capacity_bound),
    };

    // compute

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let result: ThreadCollectArbitrary<Vo::Fallibility> = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
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
