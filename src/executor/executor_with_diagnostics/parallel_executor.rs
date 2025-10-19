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
