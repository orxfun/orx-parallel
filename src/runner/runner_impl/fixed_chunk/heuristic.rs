use crate::parameters::ChunkSize;
use core::num::NonZeroUsize;

const DESIRED_MIN_CHUNK_SIZE: usize = 64;

pub fn compute_chunk_size(
    chunk_size: ChunkSize,
    initial_len: Option<usize>,
    max_num_threads: NonZeroUsize,
) -> usize {
    match chunk_size {
        ChunkSize::Auto => auto_chunk_size(initial_len, max_num_threads),
        ChunkSize::Min(min_chunk) => min_chunk_size(initial_len, max_num_threads, min_chunk.into()),
        ChunkSize::Exact(c) => c.into(),
    }
}

fn auto_chunk_size(initial_len: Option<usize>, max_num_threads: NonZeroUsize) -> usize {
    const DESIRED_CHUNK_SIZE: usize = 1024;

    match initial_len {
        None | Some(0) => 1,
        Some(initial_len) => {
            let thread_load = initial_len / max_num_threads;

            let c = match thread_load >= DESIRED_CHUNK_SIZE {
                true => {
                    let mut c = thread_load;
                    let mut diff = (c as i64 - DESIRED_CHUNK_SIZE as i64).abs();

                    while c >= DESIRED_CHUNK_SIZE {
                        let c2 = c / 2;
                        let diff2 = (c2 as i64 - DESIRED_CHUNK_SIZE as i64).abs();
                        match diff2 < diff {
                            true => (c, diff) = (c2, diff2),
                            false => break,
                        }
                    }

                    c
                }
                false => thread_load / 4,
            };

            core::cmp::max(c, 1)
        }
    }
}

fn min_chunk_size(
    initial_len: Option<usize>,
    max_num_threads: NonZeroUsize,
    min_chunk: usize,
) -> usize {
    core::cmp::max(min_chunk, auto_chunk_size(initial_len, max_num_threads))
}
