use super::shared_state::SharedStateWithDiagnostics;
use crate::{ParallelExecutor, ThreadExecutor};
use orx_concurrent_iter::ConcurrentIter;

pub struct ThreadExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    thread_idx: usize,
    executor: E::ThreadExecutor,
}

impl<E> ThreadExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    pub(super) fn new(thread_idx: usize, executor: E::ThreadExecutor) -> Self {
        Self {
            thread_idx,
            executor,
        }
    }
}

impl<E> ThreadExecutor for ThreadExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    type SharedState = SharedStateWithDiagnostics<E::SharedState>;

    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> usize
    where
        I: ConcurrentIter,
    {
        self.executor.next_chunk_size(shared_state.inner(), iter)
    }

    fn begin_chunk(&mut self, chunk_size: usize) {
        self.executor.begin_chunk(chunk_size);
    }

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize) {
        self.executor
            .complete_chunk(shared_state.inner(), chunk_size);
    }

    fn complete_task(&mut self, shared_state: &Self::SharedState) {
        self.executor.complete_task(shared_state.inner());
    }
}
