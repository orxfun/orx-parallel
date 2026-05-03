use crate::parameters::ChunkSize;

/// Default chunk size for iterators with unknown length (recursive / dynamic iterators).
///
/// A chunk size > 1 reduces atomic queue contention and gives each thread a meaningful
/// initial batch of work even before the queue has fully grown.
pub const DEFAULT_RECURSIVE_CHUNK_SIZE: usize = 64;

/// How many *visible* queue items we require per active thread before spawning the next one.
///
/// Concretely, thread `t` (0-indexed) is only spawned if
/// `queue_lower_bound >= t * MIN_ITEMS_PER_THREAD_FACTOR * chunk_size`.
///
/// A value of 2 means each thread should see at least 2 chunk-widths of work ahead.
pub const MIN_ITEMS_PER_THREAD_FACTOR: usize = 2;

pub fn compute_chunk_size(chunk_size: ChunkSize, max_num_threads: usize) -> usize {
    match chunk_size {
        ChunkSize::Auto => DEFAULT_RECURSIVE_CHUNK_SIZE,
        ChunkSize::Min(min) => {
            let min: usize = min.into();
            min.max(DEFAULT_RECURSIVE_CHUNK_SIZE)
        }
        // User-specified exact value: use as-is (they know better)
        ChunkSize::Exact(c) => c.into(),
    }
}

pub fn compute_min_items_per_thread(chunk_size: usize) -> usize {
    chunk_size * MIN_ITEMS_PER_THREAD_FACTOR
}
