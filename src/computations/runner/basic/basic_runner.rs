use super::{chunk_size::ResolvedChunkSize, num_threads::maximum_num_threads};
use crate::{
    computations::{computation_kind::ComputationKind, runner::parallel_runner::ParallelRunner},
    parameters::Params,
};
use orx_concurrent_iter::ConcurrentIter;

const LAG_PERIODICITY: usize = 4;

pub struct BasicRunner;

impl BasicRunner {
    fn spawn_new(max_num_threads: usize, num_spawned: usize, remaining: Option<usize>) -> bool {
        match (num_spawned, remaining) {
            (_, Some(0)) => false,
            (x, _) if x >= max_num_threads => false,
            _ => true,
        }
    }

    fn next_chunk(
        max_num_threads: usize,
        num_spawned: usize,
        resolved_chunk_size: ResolvedChunkSize,
        initial_len: Option<usize>,
        remaining_len: Option<usize>,
    ) -> Option<usize> {
        match (initial_len, remaining_len) {
            (Some(initial_len), Some(remaining_len)) => Self::next_chunk_size_known_len(
                max_num_threads,
                num_spawned,
                initial_len,
                resolved_chunk_size,
                remaining_len,
            ),
            _ => {
                Self::next_chunk_size_unknown_len(max_num_threads, num_spawned, resolved_chunk_size)
            }
        }
    }

    fn next_chunk_size_unknown_len(
        max_num_threads: usize,
        num_spawned: usize,
        resolved_chunk_size: ResolvedChunkSize,
    ) -> Option<usize> {
        match num_spawned {
            x if x >= max_num_threads => None,
            _ => Some(resolved_chunk_size.chunk_size()),
        }
    }

    fn next_chunk_size_known_len(
        max_num_threads: usize,
        num_spawned: usize,
        initial_len: usize,
        resolved_chunk_size: ResolvedChunkSize,
        remaining_len: usize,
    ) -> Option<usize> {
        match num_spawned {
            x if x >= max_num_threads => None,
            _ => match resolved_chunk_size {
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

impl ParallelRunner for BasicRunner {
    fn run<I, T1, TN>(
        params: Params,
        kind: ComputationKind,
        iter: &I,
        execution_by_one: &T1,
        execution_in_chunk: &TN,
    ) -> usize
    where
        I: ConcurrentIter,
        T1: Fn() + Sync,
        TN: Fn(usize) + Sync,
    {
        let initial_len = remaining_len(iter);
        let max_num_threads = maximum_num_threads(initial_len, params.num_threads);
        let resolved_chunk_size =
            ResolvedChunkSize::new(kind, initial_len, max_num_threads, params.chunk_size);

        let mut num_spawned = 0;

        std::thread::scope(|s| {
            let mut chunk = resolved_chunk_size.chunk_size();

            match chunk {
                1 => s.spawn(|| execution_by_one()),
                n => s.spawn(move || execution_in_chunk(n)),
            };
            num_spawned += 1;

            'lag_period: loop {
                for _ in 0..LAG_PERIODICITY {
                    match Self::spawn_new(max_num_threads, num_spawned, remaining_len(iter)) {
                        false => break 'lag_period,
                        true => {
                            match chunk {
                                1 => s.spawn(|| execution_by_one()),
                                n => s.spawn(move || execution_in_chunk(n)),
                            };
                            num_spawned += 1;
                        }
                    }
                }

                lag();

                match Self::next_chunk(
                    max_num_threads,
                    num_spawned,
                    resolved_chunk_size,
                    initial_len,
                    remaining_len(iter),
                ) {
                    Some(c) => chunk = c,
                    None => break 'lag_period,
                }
            }
        });

        num_spawned
    }
}

fn remaining_len<I: ConcurrentIter>(iter: &I) -> Option<usize> {
    match iter.size_hint() {
        (_, None) => None,
        (_, Some(upper)) => Some(upper),
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
