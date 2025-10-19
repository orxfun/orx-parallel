use crate::{
    ParallelExecutor,
    executor::executor_with_diagnostics::shared_state::SharedStateWithDiagnostics,
    runner::{ComputationKind, NumSpawned},
};

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

    type ThreadExecutor = ();

    fn new(
        kind: ComputationKind,
        params: crate::Params,
        initial_input_len: Option<usize>,
        max_num_threads: std::num::NonZeroUsize,
    ) -> Self {
        todo!()
    }

    fn new_shared_state(&self) -> Self::SharedState {
        todo!()
    }

    fn do_spawn_new<I>(
        &self,
        num_spawned: NumSpawned,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        I: orx_concurrent_iter::ConcurrentIter,
    {
        todo!()
    }

    fn new_thread_executor(&self, shared_state: &Self::SharedState) -> Self::ThreadExecutor {
        todo!()
    }
}
