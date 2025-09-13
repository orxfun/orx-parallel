use super::super::thread_runner_compute as thread;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Reduce};
use crate::orch::NumSpawned;
use crate::runner::ParallelRunnerCompute;
use crate::using::Using;
use crate::using::computations::{UM, UX};
use orx_concurrent_iter::ConcurrentIter;

// m

pub fn u_m<C, U, I, O, M1, Red>(
    runner: C,
    m: UM<U, I, O, M1>,
    reduce: Red,
) -> (NumSpawned, Option<O>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    Red: Fn(&mut U::Item, O, O) -> O + Sync,
{
    let (mut using, _, iter, map1) = m.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = NumSpawned::zero();
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned.into_inner());
            num_spawned.increment();
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

    let mut u = using.into_inner();
    let acc = results.into_iter().reduce(|a, b| reduce(&mut u, a, b));

    (num_spawned, acc)
}

// x

type ResultReduce<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;

pub fn u_x<C, U, I, Vo, M1, Red>(
    runner: C,
    x: UX<U, I, Vo, M1>,
    reduce: Red,
) -> (NumSpawned, ResultReduce<Vo>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    Red: Fn(&mut U::Item, Vo::Item, Vo::Item) -> Vo::Item + Sync,
{
    let (mut using, _, iter, xap1) = x.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = NumSpawned::zero();
    let result: Result<Vec<Vo::Item>, _> = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned.into_inner());
            num_spawned.increment();
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

        let mut error = None;
        while !handles.is_empty() {
            let mut finished_idx = None;
            for (h, handle) in handles.iter().enumerate() {
                if handle.is_finished() {
                    finished_idx = Some(h);
                    break;
                }
            }

            if let Some(h) = finished_idx {
                let handle = handles.remove(h);
                let result = handle.join().expect("failed to join the thread");
                match result {
                    Reduce::Done { acc: Some(acc) } => results.push(acc),
                    Reduce::StoppedByWhileCondition { acc: Some(acc) } => results.push(acc),
                    Reduce::StoppedByError { error: e } => {
                        error = Some(e);
                        break;
                    }
                    _ => {}
                }
            }
        }

        match error {
            Some(error) => Err(error),
            None => Ok(results),
        }
    });

    let mut u = using.into_inner();
    let acc = result.map(|results| results.into_iter().reduce(|a, b| reduce(&mut u, a, b)));

    (num_spawned, acc)
}
