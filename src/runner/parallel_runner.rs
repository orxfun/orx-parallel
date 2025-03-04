use super::{computation_kind::ComputationKind, thread_runner::ThreadRunner};
use crate::parameters::Params;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParallelRunner: Sized + Sync {
    type SharedState: Send + Sync;

    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    fn new(kind: ComputationKind, params: Params, initial_input_len: Option<usize>) -> Self;

    fn new_shared_state(&self) -> Self::SharedState;

    fn do_spawn_new<I>(
        &self,
        num_spawned: usize,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        I: ConcurrentIter;

    fn new_thread_runner(&self, shared_state: &Self::SharedState) -> Self::ThreadRunner;

    fn run<I, T>(&self, iter: &I, transform: &T)
    where
        I: ConcurrentIter,
        T: Fn(I::Item) + Sync,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(move || {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.run(iter, shared_state, transform);
                });
            }
        });
    }

    // fn run_with_idx<I,T>
}
