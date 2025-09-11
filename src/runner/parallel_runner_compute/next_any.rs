use crate::generic_values::Values;
use crate::generic_values::runner_results::Fallibility;
use crate::orch::Orchestrator;
use crate::runner::{ComputationKind, thread_runner_compute as thread};
use crate::{ParallelRunner, Params};
use orx_concurrent_iter::ConcurrentIter;

pub fn m<C, I, O, M1>(orchestrator: C, params: Params, iter: I, map1: M1) -> (usize, Option<O>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    let runner = orchestrator.new_runner(ComputationKind::Collect, params, iter.try_get_len());

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
                    &map1,
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

type ResultNextAny<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;

pub fn x<C, I, Vo, X1>(
    orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
) -> (usize, ResultNextAny<Vo>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
{
    let runner = orchestrator.new_runner(ComputationKind::Collect, params, iter.try_get_len());

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

        let mut result = Ok(None);
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
                match handle.join().expect("failed to join the thread") {
                    Ok(Some(x)) => {
                        result = Ok(Some(x));
                        break;
                    }
                    Err(error) => {
                        result = Err(error);
                        break;
                    }
                    Ok(None) => {}
                }
            }
        }

        result
    });

    (num_spawned, result)
}
