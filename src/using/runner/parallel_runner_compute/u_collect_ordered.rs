use super::super::thread_runner_compute as thread;
use crate::runner::ParallelRunnerCompute;
use crate::using::Using;
use crate::using::computations::{UM, UX};
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, ParallelCollect, ThreadCollect};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn u_m<C, U, I, O, M1, P>(runner: C, m: UM<U, I, O, M1>, pinned_vec: P) -> (usize, P)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let offset = pinned_vec.len();
    let (mut using, _, iter, map1) = m.destruct();

    let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            let u = using.create(num_spawned);
            num_spawned += 1;
            s.spawn(|| {
                thread::u_collect_ordered::u_m(
                    runner.new_thread_runner(shared_state),
                    u,
                    &iter,
                    shared_state,
                    &map1,
                    &o_bag,
                    offset,
                );
            });
        }
    });

    let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
    (num_spawned, values)
}

// x

pub fn u_x<C, U, I, Vo, M1, P>(
    runner: C,
    x: UX<U, I, Vo, M1>,
    pinned_vec: P,
) -> (usize, ParallelCollect<Vo, P>)
where
    C: ParallelRunnerCompute,
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let (mut using, _, iter, xap1) = x.destruct();

    // compute
    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let result: Result<Vec<ThreadCollect<Vo>>, <Vo::Fallibility as Fallibility>::Error> =
        std::thread::scope(|s| {
            let mut handles = vec![];

            while runner.do_spawn_new(num_spawned, shared_state, &iter) {
                let u = using.create(num_spawned);
                num_spawned += 1;
                handles.push(s.spawn(|| {
                    thread::u_collect_ordered::u_x(
                        runner.new_thread_runner(shared_state),
                        u,
                        &iter,
                        shared_state,
                        &xap1,
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
                    match result.into_result() {
                        Ok(result) => results.push(result),
                        Err(e) => {
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

    let result = match result {
        Err(error) => ParallelCollect::StoppedByError { error },
        Ok(results) => ParallelCollect::reduce(results, pinned_vec),
    };

    (num_spawned, result)
}
