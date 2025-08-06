use crate::runner::thread_runner_compute as thread;
use crate::{
    computations::{Values, Xfx},
    runner::ParallelRunnerCompute,
};
use orx_concurrent_iter::ConcurrentIter;

// xfx

pub fn xfx<C, I, Vt, Vo, M1, F, M2>(
    runner: C,
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
) -> (usize, Option<Vo::Item>)
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vt + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    let (_, iter, xap1, filter, xap2) = xfx.destruct();

    let state = runner.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    let results = std::thread::scope(|s| {
        let mut handles = vec![];

        while runner.do_spawn_new(num_spawned, shared_state, &iter) {
            num_spawned += 1;
            handles.push(s.spawn(|| {
                thread::next::xfx(
                    runner.new_thread_runner(shared_state),
                    &iter,
                    shared_state,
                    &xap1,
                    &filter,
                    &xap2,
                )
            }))
        }

        let mut results: Vec<(usize, Vo::Item)> = Vec::with_capacity(handles.len());
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
