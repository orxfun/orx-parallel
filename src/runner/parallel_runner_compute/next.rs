use crate::computations::{M, X};
use crate::runner::thread_runner_compute as thread;
use crate::values::runner_results::NextWithIdx;
use crate::{runner::ParallelRunnerCompute, values::Values};
use orx_concurrent_iter::ConcurrentIter;

pub fn m<C, I, O, M1>(runner: C, m: M<I, O, M1>) -> (usize, Option<O>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    let (_, iter, xap1) = m.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::next::m(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                )
            }))
        }

        let mut results: Vec<(usize, O)> = Vec::with_capacity(handles.len());
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

pub fn x<C, I, Vo, X1>(runner: C, x: X<I, Vo, X1>) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
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
                thread::next::x(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                )
            }))
        }

        let mut results: Vec<NextWithIdx<Vo::Item, ()>> = Vec::with_capacity(handles.len());
        for x in handles {
            let thread_next = x.join().expect("failed to join the thread");
            results.push(thread_next);
        }
        results
    });

    let result = NextWithIdx::reduce(results);
    let acc = result.into_found_value();

    (num_spawned, acc)
}
