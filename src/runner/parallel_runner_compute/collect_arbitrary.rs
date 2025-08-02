#[cfg(test)]
use crate::computations::{M, UsingM};
use crate::{
    computations::{UsingX, Values, X, Xfx},
    runner::{ParallelRunnerCompute, thread_runner_compute},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

#[cfg(test)]
pub fn m<C, I, O, M1, P>(runner: C, m: M<I, O, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
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
                thread_runner_compute::collect_arbitrary::m(
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

#[cfg(test)]
pub fn using_m<C, U, I, O, M1, P>(runner: C, m: UsingM<U, I, O, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    U: Send + Clone,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let (_, using, iter, map1) = m.destruct();

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
            let using = using.clone();
            s.spawn(|| {
                thread_runner_compute::collect_arbitrary::using_m(
                    runner.new_thread_runner(shared_state),
                    using,
                    &iter,
                    &shared_state,
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

pub fn x<C, I, Vo, M1, P>(runner: C, x: X<I, Vo, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
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
    std::thread::scope(|s| {
        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            s.spawn(|| {
                thread_runner_compute::collect_arbitrary::x(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                    &bag,
                );
            });
        }
    });
    let values = bag.into_inner();
    (num_spawned, values)
}

pub fn using_x<C, U, I, Vo, M1, P>(runner: C, m: UsingX<U, I, Vo, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    U: Send + Clone,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U, I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let (_, using, iter, xap1) = m.destruct();

    let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
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
            let using = using.clone();
            s.spawn(|| {
                thread_runner_compute::collect_arbitrary::using_x(
                    runner.new_thread_runner(shared_state),
                    using,
                    &iter,
                    shared_state,
                    &xap1,
                    &bag,
                );
            });
        }
    });
    let values = bag.into_inner();
    (num_spawned, values)
}

// xfx

pub fn xfx<C, I, Vt, Vo, M1, F, M2, P>(
    runner: C,
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    pinned_vec: P,
) -> (usize, P)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let capacity_bound = pinned_vec.capacity_bound();
    let offset = pinned_vec.len();
    let (_, iter, xap1, filter, xap2) = xfx.destruct();

    let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
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
                thread_runner_compute::collect_arbitrary::xfx(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                    &filter,
                    &xap2,
                    &bag,
                );
            });
        }
    });
    let values = bag.into_inner();
    (num_spawned, values)
}
