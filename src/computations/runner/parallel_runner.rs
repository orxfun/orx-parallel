use super::thread_runner::ThreadRunner;
use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, Element, Enumeration};

pub trait ParallelRunner: Sized {
    type SharedState: Send + Sync;

    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    fn new_shared_state<E, I>(kind: ComputationKind, params: Params, iter: &I) -> Self::SharedState
    where
        E: Enumeration,
        I: ConcurrentIter<E>;

    fn do_spawn_new<E, I>(num_spawned: usize, shared_state: &Self::SharedState, iter: &I) -> bool
    where
        E: Enumeration,
        I: ConcurrentIter<E>;

    fn run<E, I, T>(kind: ComputationKind, params: Params, iter: &I, transform: &T)
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
        T: Fn(<E::Element as Element>::ElemOf<I::Item>) + Sync,
    {
        let state = Self::new_shared_state(kind, params, iter);
        let shared_state = &state;
        let chunk_size = params.chunk_size;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while Self::do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(move || {
                    Self::ThreadRunner::new().run(kind, chunk_size, iter, shared_state, transform);
                });
            }
        });
    }
}
