use crate::runner::{
    ParallelRunnerCompute, ParallelTask, ParallelTaskWithIdx, thread_runner::ThreadRunnerCompute,
};
use orx_concurrent_iter::ConcurrentIter;

pub fn run<C, I, T>(c: &C, iter: &I, task: T) -> usize
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    T: ParallelTask<Item = I::Item> + Send,
{
    let state = c.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            let task = task.clone();
            s.spawn(|| {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.run(iter, shared_state, task);
            });
        }
    });
    num_spawned
}

pub fn run_with_idx<C, I, T>(c: &C, iter: &I, task: T) -> usize
where
    C: ParallelRunnerCompute,
    I: ConcurrentIter,
    T: ParallelTaskWithIdx<Item = I::Item> + Send,
{
    let state = c.new_shared_state();
    let shared_state = &state;

    let mut num_spawned = 0;
    std::thread::scope(|s| {
        while c.do_spawn_new(num_spawned, shared_state, iter) {
            num_spawned += 1;
            let task = task.clone();
            s.spawn(|| {
                let thread_runner = c.new_thread_runner(shared_state);
                thread_runner.run_with_idx(iter, shared_state, task);
            });
        }
    });
    num_spawned
}
