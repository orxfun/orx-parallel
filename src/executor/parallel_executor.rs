use super::thread_executor::ThreadExecutor;
use crate::{
    parameters::Params,
    runner::{ComputationKind, NumSpawned},
};
use core::num::NonZeroUsize;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel executor which is responsible for taking a computation defined as a composition
/// of iterator methods, spawns threads, shares tasks and returns the result of the parallel
/// execution.
pub trait ParallelExecutor: Sized + Sync + 'static + Clone {
    /// Data shared to the thread executors.
    type SharedState: Send + Sync;

    /// Thread executor that is responsible for executing the tasks allocated to a thread.
    type ThreadExecutor: ThreadExecutor<SharedState = Self::SharedState>;

    /// Creates a new parallel executor for the given computation `kind`, parallelization `params`
    /// and `initial_input_len`.
    fn new(
        kind: ComputationKind,
        params: Params,
        initial_input_len: Option<usize>,
        max_num_threads: NonZeroUsize,
    ) -> Self;

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

    /// Creates a new thread executor provided that the current parallel execution state is
    /// `shared_state`.
    fn new_thread_executor(
        &self,
        thread_idx: usize,
        shared_state: &Self::SharedState,
    ) -> Self::ThreadExecutor;

    /// Executes the finalization tasks when the entire parallel computation is completed.
    fn complete_task(self, shared_state: Self::SharedState);
}
