use crate::computations::{M, X};
use crate::runner::thread_runner_compute as thread;
use crate::{computations::Values, runner::ParallelRunnerCompute};
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
    let result = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::next_any::m(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                )
            }));
        }

        // do not wait to join other threads
        handles
            .into_iter()
            .find_map(|x| x.join().expect("failed to join the thread"))
    });

    (num_spawned, result)
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
    let result = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::next_any::x(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                )
            }));
        }

        // do not wait to join other threads
        handles
            .into_iter()
            .find_map(|x| x.join().expect("failed to join the thread"))
    });

    (num_spawned, result)
}
