use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Reduce};
use crate::orch::Orchestrator;
use crate::runner::{ComputationKind, thread_runner_compute as thread};
use crate::{ParallelRunner, Params};
use orx_concurrent_iter::ConcurrentIter;

// m

pub fn m<C, I, O, M1, Red>(
    orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
    reduce: Red,
) -> (usize, Option<O>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
    Red: Fn(O, O) -> O + Sync,
    O: Send,
{
    let runner = orchestrator.new_runner(ComputationKind::Collect, params, iter.try_get_len());

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

type ResultReduce<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;

pub fn x<C, I, Vo, X1, Red>(
    orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
    reduce: Red,
) -> (usize, ResultReduce<Vo>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
{
    let runner = orchestrator.new_runner(ComputationKind::Collect, params, iter.try_get_len());

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let result: Result<Vec<Vo::Item>, _> = std::thread::scope(|s| {
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

    let acc = result.map(|results| results.into_iter().reduce(reduce));

    (num_spawned, acc)
}
