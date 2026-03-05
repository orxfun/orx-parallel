use crate::executor::thread_executor::ThreadExecutor;
use orx_concurrent_iter::ConcurrentIter;

pub struct FixedChunkThreadExecutor {
    pub(super) chunk_size: usize,
}

impl ThreadExecutor for FixedChunkThreadExecutor {
    type SharedState = ();

    #[inline(always)]
    fn next_chunk_size<I>(&self, _: &Self::SharedState, _: &I) -> usize
    where
        I: ConcurrentIter,
    {
        self.chunk_size
    }

    fn begin_chunk(&mut self, _: usize) {}

    fn complete_chunk(&mut self, _: &Self::SharedState, _: usize) {}

    fn complete_task(&mut self, _: &Self::SharedState) {}
}
