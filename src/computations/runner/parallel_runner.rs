use super::thread_runner::ThreadRunner;
use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, Element, Enumeration};

pub trait ParallelRunner<E, I>: Sized + Sync
where
    E: Enumeration,
    I: ConcurrentIter<E>,
{
    type SharedState: Send + Sync;

    type ThreadRunner: ThreadRunner<E, I, SharedState = Self::SharedState>;

    fn new(kind: ComputationKind, params: Params, iter: &I) -> Self;

    fn new_shared_state(&self) -> Self::SharedState;

    fn do_spawn_new(num_spawned: usize, shared_state: &Self::SharedState, iter: &I) -> bool;

    fn new_thread_runner(&self) -> Self::ThreadRunner;

    fn run<T>(&self, kind: ComputationKind, params: Params, iter: &I, transform: &T)
    where
        T: Fn(<E::Element as Element>::ElemOf<I::Item>) + Sync,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while Self::do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(move || {
                    let thread_runner = self.new_thread_runner();
                    thread_runner.run(iter, shared_state, transform);
                });
            }
        });
    }
}
