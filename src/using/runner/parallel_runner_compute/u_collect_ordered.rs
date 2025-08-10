use super::super::thread_runner_compute as thread;
use crate::computations::{Values, heap_sort_into};
use crate::runner::ParallelRunnerCompute;
use crate::using::Using;
use crate::using::computations::{UM, UX};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn u_m<C, U, I, O, M1, P>(runner: C, m: UM<U, I, O, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let offset = pinned_vec.len();
    let (mut using, _, iter, map1) = m.destruct();

    let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            s.spawn(|| {
                thread::u_collect_ordered::u_m(
                    runner.new_thread_runner(shared_state),
                    u,
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

pub fn u_x<C, U, I, Vo, M1, P>(runner: C, x: UX<U, I, Vo, M1>, mut pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let (mut using, _, iter, xap1) = x.destruct();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let vectors = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::u_collect_ordered::u_x(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &xap1,
                )
            }));
        }

        let mut vectors = Vec::with_capacity(handles.len());
        for x in handles {
            vectors.push(x.join().expect("failed to join the thread"));
        }
        vectors
    });

    heap_sort_into(vectors, &mut pinned_vec);
    (num_spawned, pinned_vec)
}
