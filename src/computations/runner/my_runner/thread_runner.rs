use super::parallel_runner::MyParallelRunner;
use crate::computations::runner::thread_runner::ThreadRunner;
use orx_concurrent_iter::{ConcurrentIter, Enumeration};
use std::sync::atomic::AtomicUsize;

pub struct MyThreadRunner;

impl<E, I> ThreadRunner<E, I> for MyThreadRunner
where
    E: Enumeration,
    I: ConcurrentIter<E>,
{
    type SharedState = AtomicUsize;

    type ParallelRunner = MyParallelRunner;

    fn new() -> Self {
        Self
    }

    fn next_chunk_size(&self, shared_state: &Self::SharedState, iter: &I) -> Option<usize> {
        todo!()
    }

    fn begin_chunk(&mut self, _: usize) {}

    fn complete_chunk(&mut self, _: &Self::SharedState, chunk_size: usize) {}

    fn complete_task(&mut self, _: &Self::SharedState) {}
}
