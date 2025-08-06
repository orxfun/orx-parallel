use crate::runner::thread_runner_compute as thread;
use crate::{
    computations::{M, Values, X, Xfx, heap_sort_into},
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
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
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
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let (_, iter, xap1) = x.destruct();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let vectors = std::thread::scope(|s| {
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
        for x in handles {
            vectors.push(x.join().expect("failed to join the thread"));
        }
        vectors
    });

    heap_sort_into(vectors, &mut pinned_vec);
    (num_spawned, pinned_vec)
}

// xfx

pub fn xfx<C, I, Vt, Vo, M1, F, M2, P>(
    runner: C,
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    mut pinned_vec: P,
) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Sync,
    F: Fn(&Vt::Item) -> bool + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let (_, iter, xap1, filter, xap2) = xfx.destruct();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let vectors = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::collect_ordered::xfx(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                    &filter,
                    &xap2,
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
