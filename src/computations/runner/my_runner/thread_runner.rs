use super::parallel_runner::MyParallelRunner;
use crate::computations::runner::thread_runner::ThreadRunner;
use orx_concurrent_iter::{ConcurrentIter, Enumeration};
use std::sync::atomic::AtomicUsize;

pub struct MyThreadRunner {
    pub(super) chunk_size: usize,
}

impl<E, I> ThreadRunner<E, I> for MyThreadRunner
where
    E: Enumeration,
    I: ConcurrentIter<E>,
{
    type SharedState = AtomicUsize;

    fn next_chunk_size(&self, _: &Self::SharedState, iter: &I) -> Option<usize> {
        match iter.try_get_len() {
            Some(0) => None,
            _ => Some(self.chunk_size),
        }
    }

    fn begin_chunk(&mut self, _: usize) {}

    fn complete_chunk(&mut self, _: &Self::SharedState, chunk_size: usize) {}

    fn complete_task(&mut self, _: &Self::SharedState) {}
}
