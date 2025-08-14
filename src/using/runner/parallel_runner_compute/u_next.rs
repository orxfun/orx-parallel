use super::super::thread_runner_compute as thread;
use crate::runner::ThreadNext;
use crate::using::Using;
use crate::using::computations::{UM, UX};
use crate::{runner::ParallelRunnerCompute, values::TransformableValues};
use orx_concurrent_iter::ConcurrentIter;

pub fn u_m<C, U, I, O, M1>(runner: C, m: UM<U, I, O, M1>) -> (usize, Option<O>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
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
                thread::u_next::u_m(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &map1,
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

pub fn u_x<C, U, I, Vo, X1>(runner: C, x: UX<U, I, Vo, X1>) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
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
                thread::u_next::u_x(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &xap1,
                )
            }))
        }

        let mut results: Vec<ThreadNext<Vo::Item, ()>> = Vec::with_capacity(handles.len());
        for x in handles {
            let thread_next = x.join().expect("failed to join the thread");
            results.push(thread_next);
        }
        results
    });

    let result = ThreadNext::reduce(results);
    let acc = result.into_found_value();

    (num_spawned, acc)
}
