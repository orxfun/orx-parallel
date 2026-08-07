pub struct ChunkState {
    pub requested_chunk_size: usize,
    pub started_at_ns: u64,
}

impl ChunkState {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            requested_chunk_size: chunk_size,
            started_at_ns: now_ns(),
        }
    }

    pub fn elapsed_ns(&self) -> u64 {
        now_ns().saturating_sub(self.started_at_ns)
    }
}

#[inline(always)]
fn now_ns() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // Date.now exists in both window and worker contexts; precision is sufficient for adaptive tuning.
        return (js_sys::Date::now() * 1_000_000.0) as u64;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX_EPOCH");
        now.as_nanos().min(u64::MAX as u128) as u64
    }
}
