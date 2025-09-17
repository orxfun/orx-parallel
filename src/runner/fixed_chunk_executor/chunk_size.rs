use crate::{parameters::ChunkSize, runner::computation_kind::ComputationKind};
use core::num::NonZeroUsize;

const MAX_CHUNK_SIZE: usize = 1 << 20;
const DESIRED_MIN_CHUNK_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedChunkSize {
    Min(usize),
    Exact(usize),
}

impl ResolvedChunkSize {
    pub fn new(
        kind: ComputationKind,
        initial_len: Option<usize>,
        max_num_threads: NonZeroUsize,
        chunk_size: ChunkSize,
    ) -> Self {
        match chunk_size {
            ChunkSize::Auto => Self::Min(auto_chunk_size(kind, initial_len, max_num_threads)),
            ChunkSize::Min(x) => Self::Min(min_chunk_size(initial_len, max_num_threads, x.into())),
            ChunkSize::Exact(x) => Self::Exact(x.into()),
        }
    }

    pub fn chunk_size(self) -> usize {
        match self {
            Self::Min(x) => x,
            Self::Exact(x) => x,
        }
    }
}

const fn min_required_len(kind: ComputationKind, one_round_len: usize) -> usize {
    match kind {
        ComputationKind::Collect => one_round_len * 4,
        ComputationKind::Reduce => one_round_len * 4,
        ComputationKind::EarlyReturn => one_round_len * 8,
    }
}

fn auto_chunk_size(
    kind: ComputationKind,
    initial_len: Option<usize>,
    max_num_threads: NonZeroUsize,
) -> usize {
    fn find_chunk_size(kind: ComputationKind, input_len: usize, num_threads: usize) -> usize {
        let mut chunk_size = MAX_CHUNK_SIZE;

        loop {
            let one_round_len = chunk_size * num_threads;

            // heterogeneity buffer
            if input_len >= min_required_len(kind, one_round_len) {
                break;
            }

            // len is still greater, we don't want chunk size to be too small
            if input_len >= one_round_len && chunk_size <= DESIRED_MIN_CHUNK_SIZE {
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

    match initial_len {
        None => 1, // TODO: is this a good choice?
        Some(0) => 1,
        Some(input_len) => find_chunk_size(kind, input_len, max_num_threads.into()),
    }
}

fn min_chunk_size(
    initial_len: Option<usize>,
    max_num_threads: NonZeroUsize,
    chunk_size: usize,
) -> usize {
    let max_num_threads: usize = max_num_threads.into();
    match initial_len {
        None => chunk_size,
        Some(0) => 1,
        Some(len) => {
            let one_round_len = max_num_threads * chunk_size;
            match one_round_len > len {
                true => div_ceil(len, max_num_threads),
                false => chunk_size,
            }
        }
    }
}

const fn div_ceil(number: usize, divider: usize) -> usize {
    let x = number / divider;
    let remainder = number - x * divider;
    x + if remainder > 0 { 1 } else { 0 }
}
