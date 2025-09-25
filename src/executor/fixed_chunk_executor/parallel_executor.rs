use super::{chunk_size::ResolvedChunkSize, thread_executor::FixedChunkThreadExecutor};
use crate::runner::ComputationKind;
use crate::{
    executor::parallel_executor::ParallelExecutor, parameters::Params, runner::NumSpawned,
};
use core::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
use orx_concurrent_iter::ConcurrentIter;

const LAG_PERIODICITY: usize = 4;

pub struct FixedChunkRunner {
    initial_len: Option<usize>,
    resolved_chunk_size: ResolvedChunkSize,
    max_num_threads: usize,
    current_chunk_size: AtomicUsize,
}

impl FixedChunkRunner {
    fn spawn_new(&self, num_spawned: usize, remaining: Option<usize>) -> bool {
        match (num_spawned, remaining) {
            (_, Some(0)) => false,
            (x, _) if x >= self.max_num_threads => false,
            _ => true,
        }
    }

    fn next_chunk(&self, num_spawned: usize, remaining_len: Option<usize>) -> Option<usize> {
        match (self.initial_len, remaining_len) {
            (Some(initial_len), Some(remaining_len)) => {
                self.next_chunk_size_known_len(num_spawned, initial_len, remaining_len)
            }
            _ => self.next_chunk_size_unknown_len(num_spawned),
        }
    }

    fn next_chunk_size_unknown_len(&self, num_spawned: usize) -> Option<usize> {
        match num_spawned {
            x if x >= self.max_num_threads => None,
            _ => Some(self.resolved_chunk_size.chunk_size()),
        }
    }

    fn next_chunk_size_known_len(
        &self,
        num_spawned: usize,
        initial_len: usize,
        remaining_len: usize,
    ) -> Option<usize> {
        match num_spawned {
            x if x >= self.max_num_threads => None,
            _ => match self.resolved_chunk_size {
                ResolvedChunkSize::Exact(x) => Some(x),
                ResolvedChunkSize::Min(x) => {
                    let chunk_size = match num_spawned {
                        0 => x,
                        _ => {
                            let done = initial_len - remaining_len;
                            let done_per_thread = done / num_spawned;
                            let num_chunks_per_thread = (done_per_thread / x).max(1);
                            let num_chunks_per_thread = num_chunks_per_thread.max(1);
                            num_chunks_per_thread * x
                        }
                    };

                    Some(chunk_size)
                }
            },
        }
    }
}

impl ParallelExecutor for FixedChunkRunner {
    type SharedState = ();

    type ThreadExecutor = FixedChunkThreadExecutor;

    fn new(
        kind: ComputationKind,
        params: Params,
        initial_len: Option<usize>,
        max_num_threads: NonZeroUsize,
    ) -> Self {
        let resolved_chunk_size =
            ResolvedChunkSize::new(kind, initial_len, max_num_threads, params.chunk_size);

        Self {
            initial_len,
            resolved_chunk_size,
            max_num_threads: max_num_threads.into(),
            current_chunk_size: resolved_chunk_size.chunk_size().into(),
        }
    }

    fn new_shared_state(&self) -> Self::SharedState {}

    fn do_spawn_new<I>(&self, num_spawned: NumSpawned, _: &Self::SharedState, iter: &I) -> bool
    where
        I: ConcurrentIter,
    {
        let num_spawned = num_spawned.into_inner();
        if num_spawned.is_multiple_of(LAG_PERIODICITY) {
            match self.next_chunk(num_spawned, iter.try_get_len()) {
                Some(c) => self.current_chunk_size.store(c, Ordering::Relaxed),
                None => return false,
            }
        }

        self.spawn_new(num_spawned, iter.try_get_len())
    }

    fn new_thread_executor(&self, _: &Self::SharedState) -> Self::ThreadExecutor {
        Self::ThreadExecutor {
            chunk_size: self.current_chunk_size.load(Ordering::Relaxed),
        }
    }
}
