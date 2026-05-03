use crate::parameters::ChunkSize;

/// Default chunk size for iterators with unknown length (recursive / dynamic iterators).
///
/// A chunk size > 1 reduces atomic queue contention and gives each thread a meaningful
/// initial batch of work even before the queue has fully grown.
pub const DEFAULT_RECURSIVE_CHUNK_SIZE: usize = 64;

/// Cap used for unknown-length recursive workloads when `num_threads` is `Auto`.
/// Empirically, capping avoids heavy spawn overhead and run-to-run variance that appears
/// when creating all available threads eagerly for small initial frontiers.
pub const MAX_RECURSIVE_AUTO_THREADS: usize = 16;

pub fn compute_chunk_size(chunk_size: ChunkSize, _max_num_threads: usize) -> usize {
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
