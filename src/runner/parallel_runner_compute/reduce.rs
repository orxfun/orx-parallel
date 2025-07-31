use crate::{
    computations::Values,
    runner::{ParallelRunnerCompute, thread_runner::ThreadRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;

pub fn x_reduce<C, I, Vo, M1, X>(
    c: &C,
    iter: &I,
    map1: &M1,
    reduce: &X,
) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    let state = c.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            handles.push(s.spawn(move || {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.x_reduce(iter, shared_state, map1, reduce)
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

pub fn xfx_reduce<C, I, Vt, Vo, M1, F, M2, X>(
    c: &C,
    iter: &I,
    map1: &M1,
    filter: &F,
    map2: &M2,
    reduce: &X,
) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    let state = c.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            handles.push(s.spawn(move || {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.xfx_reduce(iter, shared_state, map1, filter, map2, reduce)
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
