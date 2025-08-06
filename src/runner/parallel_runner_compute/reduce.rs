use crate::computations::{M, X, Xfx};
use crate::runner::thread_runner_compute as thread;
use crate::{computations::Values, runner::ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

// m

pub fn m<C, I, O, M1, Red>(runner: C, m: M<I, O, M1>, reduce: Red) -> (usize, Option<O>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    Red: Fn(O, O) -> O + Sync,
{
    let (_, iter, map1) = m.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::reduce::m(
                    runner.new_thread_runner(shared_state),
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

pub fn x<C, I, Vo, M1, Red>(runner: C, x: X<I, Vo, M1>, reduce: Red) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
{
    let (_, iter, xap1) = x.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::reduce::x(
                    runner.new_thread_runner(shared_state),
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

pub fn xfx<C, I, Vt, Vo, M1, F, M2, Red>(
    runner: C,
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    reduce: Red,
) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vt + Sync,
    F: Fn(&Vt::Item) -> bool + Sync,
    M2: Fn(Vt::Item) -> Vo + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
{
    let (_, iter, xap1, filter, xap2) = xfx.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::reduce::xfx(
                    runner.new_thread_runner(shared_state),
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
