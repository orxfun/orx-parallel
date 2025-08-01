use crate::{
    computations::{M, UsingM},
    runner::{ParallelRunnerCompute, thread_runner_compute::ThreadRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn m_collect_ordered<C, I, O, M1, P>(runner: C, m: M<I, O, M1>, pinned_vec: P) -> (usize, P)
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
                let thread_runner = runner.new_thread_runner(shared_state);
                thread_runner.m_collect_ordered(&iter, shared_state, &map1, &o_bag, offset);
            });
        }
    });

    let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
    (num_spawned, values)
}

pub fn using_m_collect_ordered<C, U, I, O, M1, P>(
    runner: C,
    m: UsingM<U, I, O, M1>,
    pinned_vec: P,
) -> (usize, P)
where
    C: ParallelRunnerCompute,
    U: Clone + Send,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let offset = pinned_vec.len();
    let (_, using, iter, map1) = m.destruct();

    let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            let using = using.clone();
            s.spawn(|| {
                let thread_runner = runner.new_thread_runner(shared_state);
                thread_runner.using_m_collect_ordered(
                    using,
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
