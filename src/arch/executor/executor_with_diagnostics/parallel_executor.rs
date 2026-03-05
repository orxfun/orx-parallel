use super::{
    shared_state::SharedStateWithDiagnostics, thread_executor::ThreadExecutorWithDiagnostics,
};
use crate::ParallelExecutor;
use crate::runner::{ComputationKind, NumSpawned};
use orx_concurrent_iter::ConcurrentIter;
use std::num::NonZeroUsize;

/// A parallel executor which wraps another parallel executor `E` and collects diagnostics about:
///
/// * how many threads are used for the parallel computation
/// * how many times each thread received a tasks
/// * average chunk size; i.e., average number of tasks, that each thread received
/// * and finally, explicit chunk sizes for the first task assignments.
///
/// The diagnostics are printed on the stdout once the parallel computation is completed.
/// Therefore, this executor is suitable only for test purposed, but not for production.
///
/// Any executor can be converted into executor with diagnostics.
/// In the example below, executor of the default runner is converted to executor with diagnostics.
///
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // normal execution
///
/// let range = 0..4096;
/// let sum = range
///     .par()
///     .map(|x| x + 1)
///     .filter(|x| x.is_multiple_of(2))
///     .sum();
/// assert_eq!(sum, 4196352);
///
/// // execution with diagnostics
///
/// let range = 0..4096;
/// let sum = range
///     .par()
///     .with_runner(DefaultRunner::default().with_diagnostics())
///     .map(|x| x + 1)
///     .filter(|x| x.is_multiple_of(2))
///     .sum();
/// assert_eq!(sum, 4196352);
///
/// // prints diagnostics, which looks something like the following:
/// //
/// // - Number of threads used = 5
/// //
/// // - [Thread idx]: num_calls, num_tasks, avg_chunk_size, first_chunk_sizes
/// //   - [0]: 25, 1600, 64, [64, 64, 64, 64, 64, 64, 64, 64, 64, 64]
/// //   - [1]: 26, 1664, 64, [64, 64, 64, 64, 64, 64, 64, 64, 64, 64]
/// //   - [2]: 13, 832, 64, [64, 64, 64, 64, 64, 64, 64, 64, 64, 64]
/// //   - [3]: 0, 0, 0, []
/// //   - [4]: 0, 0, 0, []
/// ```
#[derive(Clone)]
pub struct ParallelExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    executor: E,
}

impl<E> ParallelExecutor for ParallelExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    type SharedState = SharedStateWithDiagnostics<E::SharedState>;

    type ThreadExecutor = ThreadExecutorWithDiagnostics<E>;

    fn new(
        kind: ComputationKind,
        params: crate::Params,
        initial_input_len: Option<usize>,
        max_num_threads: NonZeroUsize,
    ) -> Self {
        let executor = E::new(kind, params, initial_input_len, max_num_threads);
        Self { executor }
    }

    fn new_shared_state(&self) -> Self::SharedState {
        let inner_state = self.executor.new_shared_state();
        SharedStateWithDiagnostics::new(inner_state)
    }

    fn do_spawn_new<I>(
        &self,
        num_spawned: NumSpawned,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        I: ConcurrentIter,
    {
        self.executor
            .do_spawn_new(num_spawned, shared_state.inner(), iter)
    }

    fn new_thread_executor(
        &self,
        thread_idx: usize,
        shared_state: &Self::SharedState,
    ) -> Self::ThreadExecutor {
        let executor = self
            .executor
            .new_thread_executor(thread_idx, shared_state.inner());
        ThreadExecutorWithDiagnostics::new(thread_idx, executor)
    }

    fn complete_task(self, shared_state: Self::SharedState) {
        shared_state.display();
    }
}
