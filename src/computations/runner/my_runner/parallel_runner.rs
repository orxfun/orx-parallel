use super::{
    chunk_size::ResolvedChunkSize, num_threads::maximum_num_threads, thread_runner::MyThreadRunner,
};
use crate::{
    computations::{computation_kind::ComputationKind, runner::parallel_runner::ParallelRunner},
    parameters::Params,
};
use orx_concurrent_iter::{ConcurrentIter, Enumeration};
use std::sync::atomic::{AtomicUsize, Ordering};

const LAG_PERIODICITY: usize = 4;

pub struct MyParallelRunner {
    initial_len: Option<usize>,
    resolved_chunk_size: ResolvedChunkSize,
    max_num_threads: usize,
}

impl MyParallelRunner {
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

    fn do_spawn_new(&self, num_spawned: usize, shared_state: &Self::SharedState, iter: &I) -> bool {
        if num_spawned % LAG_PERIODICITY == 0 {
            lag();
            match self.next_chunk(num_spawned, iter.try_get_len()) {
                Some(c) => shared_state.store(c, Ordering::Relaxed),
                None => return false,
            }
        }

        self.spawn_new(num_spawned, iter.try_get_len())
    }

    fn new_thread_runner(&self, shared_state: &Self::SharedState) -> Self::ThreadRunner {
        Self::ThreadRunner {
            chunk_size: shared_state.load(Ordering::Relaxed),
        }
    }
}

fn lag() {
    fn fibonacci(n: i32) -> i32 {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            let c = i32::saturating_add(a, b);
            a = b;
            b = c;
        }
        a
    }
    assert!(std::hint::black_box(fibonacci(1 << 16)) > 0);
}
