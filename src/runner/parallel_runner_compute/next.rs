use crate::ParallelRunner;
use crate::computational_variants::{ParMap, ParXap};
use crate::generic_values::TransformableValues;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Infallible, NextSuccess, NextWithIdx};
use crate::orch::Orchestrator;
use crate::runner::{ComputationKind, thread_runner_compute as thread};
use orx_concurrent_iter::ConcurrentIter;

pub fn m<C, I, O, M1>(m: ParMap<I, O, M1, C>) -> (usize, Option<O>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    let (orchestrator, params, iter, map1) = m.destruct();
    let runner = orchestrator.new_runner(ComputationKind::Collect, params, iter.try_get_len());

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

type ResultNext<Vo> = Result<
    Option<(usize, <Vo as Values>::Item)>,
    <<Vo as Values>::Fallibility as Fallibility>::Error,
>;

pub fn x<C, I, Vo, X1>(x: ParXap<I, Vo, X1, C>) -> (usize, ResultNext<Vo>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
{
    let (orchestrator, params, iter, xap1) = x.destruct();
    let runner = orchestrator.new_runner(ComputationKind::Collect, params, iter.try_get_len());

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

    let next = result.map(NextSuccess::reduce);

    (num_spawned, next)
}
