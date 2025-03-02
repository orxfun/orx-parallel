use super::{
    chunk_size::ResolvedChunkSize, num_threads::maximum_num_threads, thread_runner::MyThreadRunner,
};
use crate::{
    computations::{computation_kind::ComputationKind, runner::parallel_runner::ParallelRunner},
    parameters::Params,
};
use orx_concurrent_iter::{ConcurrentIter, Enumeration};
use std::sync::atomic::AtomicUsize;

pub struct MyParallelRunner {
    initial_len: Option<usize>,
    resolved_chunk_size: ResolvedChunkSize,
    max_num_threads: usize,
}

impl<E, I> ParallelRunner<E, I> for MyParallelRunner
where
    E: Enumeration,
    I: ConcurrentIter<E>,
{
    type SharedState = AtomicUsize;

    type ThreadRunner = MyThreadRunner;

    fn new(kind: ComputationKind, params: Params, iter: &I) -> Self {
        let initial_len = iter.try_get_len();
        let max_num_threads = maximum_num_threads(initial_len, params.num_threads);
        let resolved_chunk_size =
            ResolvedChunkSize::new(kind, initial_len, max_num_threads, params.chunk_size);

        Self {
            initial_len,
            resolved_chunk_size,
            max_num_threads,
        }
    }

    fn new_shared_state(&self) -> Self::SharedState {
        self.resolved_chunk_size.chunk_size().into()
    }

    fn do_spawn_new(num_spawned: usize, shared_state: &Self::SharedState, iter: &I) -> bool {
        todo!()
    }
}
