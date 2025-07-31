use crate::{
    computations::Values,
    runner::{ParallelRunnerCompute, thread_runner::ThreadRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;

pub fn xfx_next<C, I, Vt, Vo, M1, F, M2>(
    c: &C,
    iter: &I,
    map1: &M1,
    filter: &F,
    map2: &M2,
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
                thread_runner.xfx_next(iter, shared_state, map1, filter, map2)
            }));
        }

        let mut results: Vec<(usize, Vo::Item)> = Vec::with_capacity(handles.len());
        for x in handles {
            if let Some(x) = x.join().expect("failed to join the thread") {
                results.push(x);
            }
        }
        results
    });

    let acc = results.into_iter().min_by_key(|x| x.0).map(|x| x.1);

    (num_spawned, acc)
}

pub fn xfx_next_any<C, I, Vt, Vo, M1, F, M2>(
    c: &C,
    iter: &I,
    map1: &M1,
    filter: &F,
    map2: &M2,
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
{
    let state = c.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let result = std::thread::scope(|s| {
        let mut handles = vec![];

        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            handles.push(s.spawn(move || {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.xfx_next_any(iter, shared_state, map1, filter, map2)
            }));
        }

        // do not wait to join other threads
        handles
            .into_iter()
            .find_map(|x| x.join().expect("failed to join the thread"))
    });

    (num_spawned, result)
}
