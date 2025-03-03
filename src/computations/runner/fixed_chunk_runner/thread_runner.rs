use crate::computations::runner::thread_runner::ThreadRunner;
use orx_concurrent_iter::{ConcurrentIter, Enumeration};

pub struct FixedChunkThreadRunner {
    pub(super) chunk_size: usize,
}

impl ThreadRunner for FixedChunkThreadRunner {
    type SharedState = ();

    fn next_chunk_size<E, I>(&self, _: &Self::SharedState, iter: &I) -> Option<usize>
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
    {
        match iter.try_get_len() {
            Some(0) => None,
            _ => Some(self.chunk_size),
        }
    }

    fn begin_chunk(&mut self, _: usize) {}

    fn complete_chunk(&mut self, _: &Self::SharedState, _: usize) {}

    fn complete_task(&mut self, _: &Self::SharedState) {}
}
