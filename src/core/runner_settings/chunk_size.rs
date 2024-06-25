use crate::{
    chunk_size::{ChunkSize, ResolvedChunkSize},
    core::{runner::ParTask, runner_settings::utils::div_ceil},
};
use std::cmp::Ordering;

pub fn calc_chunk_size(
    task: ParTask,
    input_len: Option<usize>,
    max_num_threads: usize,
    chunk_size: ChunkSize,
) -> ResolvedChunkSize {
    match chunk_size {
        ChunkSize::Auto => {
            ResolvedChunkSize::Min(auto_chunk_size(task, input_len, max_num_threads))
        }
        ChunkSize::Min(x) => {
            ResolvedChunkSize::Min(min_chunk_size(input_len, max_num_threads, x.into()))
        }
        ChunkSize::Exact(x) => ResolvedChunkSize::Exact(x.into()),
    }
    .validate()
}

const INITIAL_CHUNK_SIZE: usize = 1 << 20;

const DESIRED_MIN_CHUNK_SIZE: usize = 64;

const fn min_required_len(task: ParTask, one_round_len: usize) -> usize {
    match task {
        ParTask::Collect => one_round_len * 4,
        ParTask::Reduce => one_round_len * 4,
        ParTask::EarlyReturn => one_round_len * 8,
    }
}

fn auto_chunk_size(task: ParTask, input_len: Option<usize>, max_num_threads: usize) -> usize {
    fn find_chunk_size(task: ParTask, len: usize, num_threads: usize) -> usize {
        let mut chunk_size = INITIAL_CHUNK_SIZE;

        loop {
            let one_round_len = chunk_size * num_threads;

            // heterogeneity buffer
            if len >= min_required_len(task, one_round_len) {
                break;
            }

            // len is still greater, we don't want chunk size to be too small
            if len >= one_round_len && chunk_size <= DESIRED_MIN_CHUNK_SIZE {
                break;
            }

            // absolute breaking condition
            if chunk_size == 1 {
                break;
            }

            chunk_size >>= 1;
        }

        chunk_size
    }

    match input_len {
        None => 1,
        Some(0) => 1,
        Some(len) => find_chunk_size(task, len, max_num_threads),
    }
}

fn min_chunk_size(input_len: Option<usize>, max_num_threads: usize, chunk_size: usize) -> usize {
    match input_len {
        None => chunk_size,
        Some(0) => 1,
        Some(len) => {
            let one_round_len = max_num_threads * chunk_size;
            match one_round_len.cmp(&len) {
                Ordering::Greater => div_ceil(len, max_num_threads),
                _ => chunk_size,
            }
        }
    }
}
