use super::{computation_kind::ComputationKind, thread_runner::ThreadRunner};
use crate::{orch::NumSpawned, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel runner which is responsible for taking a computation defined as a composition
/// of iterator methods, spawns threads, shares tasks and returns the result of the parallel
/// execution.
pub trait ParallelRunner: Sized + Sync + 'static {
    /// Data shared to the thread runners.
    type SharedState: Send + Sync;

    /// Thread runner that is responsible for executing the tasks allocated to a thread.
    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    /// Creates a new parallel runner for the given computation `kind`, parallelization `params`
    /// and `initial_input_len`.
    fn new(kind: ComputationKind, params: Params, initial_input_len: Option<usize>) -> Self;

    /// Creates an initial shared state.
    fn new_shared_state(&self) -> Self::SharedState;

    /// Returns true if it is beneficial to spawn a new thread provided that:
    ///
    /// * `num_spawned` threads are already been spawned, and
    /// * `shared_state` is the current parallel execution state.
    fn do_spawn_new<I>(
        &self,
        num_spawned: NumSpawned,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        I: ConcurrentIter;

    /// Creates a new thread runner provided that the current parallel execution state is
    /// `shared_state`.
    fn new_thread_runner(&self, shared_state: &Self::SharedState) -> Self::ThreadRunner;
}
