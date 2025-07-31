use crate::{
    computations::Values,
    runner::{ParallelRunnerCompute, thread_runner::ThreadRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;

pub fn x_collect_with_idx<C, I, Vo, M1>(
    c: &C,
    iter: &I,
    map1: &M1,
) -> (usize, Vec<Vec<(usize, Vo::Item)>>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    let state = c.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let vectors = std::thread::scope(|s| {
        let mut handles = vec![];

        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            handles.push(s.spawn(move || {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.x_collect_with_idx(iter, shared_state, map1)
            }));
        }

        let mut vectors = Vec::with_capacity(handles.len());
        for x in handles {
            vectors.push(x.join().expect("failed to join the thread"));
        }
        vectors
    });

    (num_spawned, vectors)
}

pub fn xfx_collect_with_idx<C, I, Vt, Vo, M1, F, M2>(
    c: &C,
    iter: &I,
    map1: &M1,
    filter: &F,
    map2: &M2,
) -> (usize, Vec<Vec<(usize, Vo::Item)>>)
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
    let vectors = std::thread::scope(|s| {
        let mut handles = vec![];

        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            handles.push(s.spawn(move || {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.xfx_collect_with_idx(iter, shared_state, map1, filter, map2)
            }));
        }

        let mut vectors = Vec::with_capacity(handles.len());
        for x in handles {
            vectors.push(x.join().expect("failed to join the thread"));
        }
        vectors
    });

    (num_spawned, vectors)
}
