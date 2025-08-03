use crate::computations::{UM, UX, UXfx};
use crate::runner::thread_runner_compute as thread;
use crate::using::Using;
use crate::{computations::Values, runner::ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

// m

pub fn u_m<C, U, I, O, M1, Red>(runner: C, m: UM<U, I, O, M1>, reduce: Red) -> (usize, Option<O>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
    Red: Fn(O, O) -> O + Send + Sync,
{
    let (mut using, _, iter, map1) = m.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::u_reduce::u_m(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &map1,
                    &reduce,
                )
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for x in handles {
            if let Some(x) = x.join().expect("failed to join the thread") {
                results.push(x);
            }
        }
        results
    });

    let acc = results.into_iter().reduce(reduce);

    (num_spawned, acc)
}

// x

pub fn u_x<C, U, I, Vo, M1, Red>(
    runner: C,
    x: UX<U, I, Vo, M1>,
    reduce: Red,
) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    let (mut using, _, iter, xap1) = x.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::u_reduce::u_x(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &xap1,
                    &reduce,
                )
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for x in handles {
            if let Some(x) = x.join().expect("failed to join the thread") {
                results.push(x);
            }
        }
        results
    });

    let acc = results.into_iter().reduce(reduce);

    (num_spawned, acc)
}

// xfx

pub fn u_xfx<C, U, I, Vt, Vo, M1, F, M2, Red>(
    runner: C,
    xfx: UXfx<U, I, Vt, Vo, M1, F, M2>,
    reduce: Red,
) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    let (mut using, _, iter, xap1, filter, xap2) = xfx.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::u_reduce::u_xfx(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &xap1,
                    &filter,
                    &xap2,
                    &reduce,
                )
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for x in handles {
            if let Some(x) = x.join().expect("failed to join the thread") {
                results.push(x);
            }
        }
        results
    });

    let acc = results.into_iter().reduce(reduce);

    (num_spawned, acc)
}
