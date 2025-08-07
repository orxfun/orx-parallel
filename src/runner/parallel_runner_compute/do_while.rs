use crate::runner::thread_runner_compute as thread;
use crate::{
    computations::{M, Values, X, Xfx, heap_sort_into},
    runner::ParallelRunnerCompute,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn m<C, I, O, E, M1>(
    runner: C,
    m: M<I, Result<O, E>, M1>,
    action: impl Fn(usize, O) + Sync,
) -> Vec<Option<(usize, E)>>
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> Result<O, E> + Sync,
    O: Send,
    E: Send,
{
    let (_, iter, map1) = m.destruct();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::do_while::m(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &map1,
                    &action,
                )
            }));
        }

        let mut results: Vec<Option<(usize, E)>> = Vec::with_capacity(handles.len());
        for x in handles {
            results.push(x.join().expect("failed to join the thread"));
        }
        results
    })
}
