use orx_concurrent_iter::ConcurrentIter;

use super::{
    shared_state::SharedStateWithDiagnostics, thread_executor::ThreadExecutorWithDiagnostics,
};
use crate::ParallelExecutor;
use crate::runner::{ComputationKind, NumSpawned};
use std::num::NonZeroUsize;

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

    fn new_thread_executor(&self, shared_state: &Self::SharedState) -> Self::ThreadExecutor {
        let executor = self.executor.new_thread_executor(shared_state.inner());
        ThreadExecutorWithDiagnostics::new(executor)
    }
}
