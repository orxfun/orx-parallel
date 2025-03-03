use super::thread_runner::ThreadRunner;
use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, Element, Enumeration};

pub trait ParallelRunner: Sized + Sync {
    type SharedState: Send + Sync;

    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    fn new<E, I>(kind: ComputationKind, params: Params, iter: &I) -> Self
    where
        E: Enumeration,
        I: ConcurrentIter<E>;

    fn new_shared_state(&self) -> Self::SharedState;

    fn do_spawn_new<E, I>(
        &self,
        num_spawned: usize,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        E: Enumeration,
        I: ConcurrentIter<E>;

    fn new_thread_runner(&self, shared_state: &Self::SharedState) -> Self::ThreadRunner;

    fn run<E, I, T>(&self, iter: &I, transform: &T)
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
        T: Fn(<E::Element as Element>::ElemOf<I::Item>) + Sync,
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
}
