use crate::computations::{M, X};
use crate::runner::thread_runner_compute as thread;
use crate::generic_values::runner_results::{Fallibility, NextSuccess, NextWithIdx};
use crate::{runner::ParallelRunnerCompute, generic_values::Values};
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

pub fn x<C, I, Vo, X1>(
    runner: C,
    x: X<I, Vo, X1>,
) -> (
    usize,
    Result<Option<(usize, Vo::Item)>, <Vo::Fallibility as Fallibility>::Error>,
)
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
    let result: Result<Vec<NextSuccess<Vo::Item>>, _> = std::thread::scope(|s| {
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
                    NextWithIdx::Found { idx, value } => {
                        results.push(NextSuccess::Found { idx, value })
                    }
                    NextWithIdx::NotFound => {}
                    NextWithIdx::StoppedByWhileCondition { idx } => {
                        results.push(NextSuccess::StoppedByWhileCondition { idx });
                    }
                    NextWithIdx::StoppedByError { error: e } => {
                        error = Some(e);
                        break;
                    }
                }
            }
        }

        match error {
            Some(error) => Err(error),
            None => Ok(results),
        }
    });

    let next = result.map(|results| NextSuccess::reduce(results));

    (num_spawned, next)
}
