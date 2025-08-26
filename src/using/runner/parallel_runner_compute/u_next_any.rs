use super::super::thread_runner_compute as thread;
use crate::using::Using;
use crate::using::computations::{UM, UX};
use crate::generic_values::runner_results::Fallibility;
use crate::{runner::ParallelRunnerCompute, generic_values::Values};
use orx_concurrent_iter::ConcurrentIter;

pub fn u_m<C, U, I, O, M1>(runner: C, m: UM<U, I, O, M1>) -> (usize, Option<O>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    let (mut using, _, iter, xap1) = m.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let result = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::u_next_any::u_m(
                    runner.new_thread_runner(shared_state),
                    u,
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

pub fn u_x<C, U, I, Vo, X1>(
    runner: C,
    x: UX<U, I, Vo, X1>,
) -> (
    usize,
    Result<Option<Vo::Item>, <Vo::Fallibility as Fallibility>::Error>,
)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    let (mut using, _, iter, xap1) = x.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let result = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::u_next_any::u_x(
                    runner.new_thread_runner(shared_state),
                    u,
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
