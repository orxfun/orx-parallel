use crate::{
    ParallelExecutor, ThreadExecutor,
    executor::executor_with_diagnostics::shared_state::SharedStateWithDiagnostics,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct ThreadExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    executor: E::ThreadExecutor,
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
        todo!()
    }

    fn begin_chunk(&mut self, chunk_size: usize) {
        todo!()
    }

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize) {
        todo!()
    }

    fn complete_task(&mut self, shared_state: &Self::SharedState) {
        todo!()
    }
}
