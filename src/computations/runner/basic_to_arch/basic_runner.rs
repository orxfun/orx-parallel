use super::{chunk_size::ResolvedChunkSize, num_threads::maximum_num_threads};
use crate::{
    computations::{
        computation_kind::ComputationKind, runner::parallel_runner_to_arch::ParallelRunnerToArchive,
    },
    parameters::Params,
};
use orx_concurrent_iter::{ConcurrentIter, Enumeration};

const LAG_PERIODICITY: usize = 4;

pub struct BasicRunnerToArch {
    initial_len: Option<usize>,
    resolved_chunk_size: ResolvedChunkSize,
    max_num_threads: usize,
}

impl BasicRunnerToArch {
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

impl ParallelRunnerToArchive for BasicRunnerToArch {
    fn new(kind: ComputationKind, params: Params, initial_len: Option<usize>) -> Self {
        let max_num_threads = maximum_num_threads(initial_len, params.num_threads);
        let resolved_chunk_size =
            ResolvedChunkSize::new(kind, initial_len, max_num_threads, params.chunk_size);

        Self {
            initial_len,
            resolved_chunk_size,
            max_num_threads,
        }
    }

    fn run<I, E, R>(&self, iter: &I, execute: &R) -> usize
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
        R: Fn(usize) + Sync,
    {
        let mut num_spawned = 0;

        std::thread::scope(|s| {
            let mut chunk = self.resolved_chunk_size.chunk_size();

            s.spawn(move || execute(chunk));
            num_spawned += 1;

            'lag_period: loop {
                for _ in 0..LAG_PERIODICITY {
                    match self.spawn_new(num_spawned, iter.try_get_len()) {
                        false => break 'lag_period,
                        true => {
                            s.spawn(move || execute(chunk));
                            num_spawned += 1;
                        }
                    }
                }

                lag();

                match self.next_chunk(num_spawned, iter.try_get_len()) {
                    Some(c) => chunk = c,
                    None => break 'lag_period,
                }
            }
        });

        num_spawned
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
