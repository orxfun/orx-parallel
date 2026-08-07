use super::timing::{Instant, Timing};

pub struct ChunkState {
    pub requested_chunk_size: usize,
    pub started_at: Instant,
}

impl ChunkState {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            requested_chunk_size: chunk_size,
            started_at: Timing::now(),
        }
    }

    pub fn elapsed_ns(&self) -> u64 {
        Timing::elapsed_ns_from(self.started_at)
    }
}
