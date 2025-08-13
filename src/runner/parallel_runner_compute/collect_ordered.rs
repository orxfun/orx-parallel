use crate::computations::min_opt_idx;
use crate::runner::thread_runner_compute as thread;
use crate::values::Values;
use crate::{
    computations::{M, X, heap_sort_into},
    runner::ParallelRunnerCompute,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn m<C, I, O, M1, P>(runner: C, m: M<I, O, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let offset = pinned_vec.len();
    let (_, iter, map1) = m.destruct();

    let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            s.spawn(|| {
                thread::collect_ordered::m(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &map1,
                    &o_bag,
                    offset,
                );
            });
        }
    });

    let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
    (num_spawned, values)
}

// x

pub fn x<C, I, Vo, M1, P>(runner: C, x: X<I, Vo, M1>, mut pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let (_, iter, xap1) = x.destruct();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let (vectors, max_idx_exc) = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::collect_ordered::x(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                )
            }));
        }

        let mut vectors = Vec::with_capacity(handles.len());
        let mut max_idx_exc = None;
        for x in handles {
            let (vector, max_idx) = x.join().expect("failed to join the thread");
            vectors.push(vector);
            max_idx_exc = min_opt_idx(max_idx_exc, max_idx);
        }
        (vectors, max_idx_exc)
    });

    heap_sort_into(vectors, max_idx_exc, &mut pinned_vec);
    (num_spawned, pinned_vec)
}
