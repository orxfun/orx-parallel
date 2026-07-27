use std::time::Instant;

pub struct ChunkState {
    pub requested_chunk_size: usize,
    pub started_at: Instant,
}

impl ChunkState {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            requested_chunk_size: chunk_size,
            started_at: Instant::now(),
        }
    }

    pub fn elapsed_ns(&self) -> u64 {
        self.started_at.elapsed().as_nanos().min(u64::MAX as u128) as u64
    }
}
