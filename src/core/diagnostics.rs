pub trait ParThreadLogger {
    fn new(chunk_size: usize) -> Self;

    fn next_chunk(&self, chunk_size: usize);

    fn log_num_spawned(num_spawned: usize);
}

pub struct NoLogger;
impl ParThreadLogger for NoLogger {
    #[inline(always)]
    fn new(_: usize) -> Self {
        Self
    }

    #[inline(always)]
    fn next_chunk(&self, _: usize) {}

    fn log_num_spawned(_: usize) {}
}

#[cfg(feature = "with_diagnostics")]
#[derive(Debug)]
pub struct ParLogger {
    begin_instant: std::time::Instant,
}

#[cfg(feature = "with_diagnostics")]
impl ParLogger {
    fn elapsed(&self) -> u128 {
        self.begin_instant.elapsed().as_nanos()
    }

    fn log_event(&self, event: &str) {
        println!("[PAR | @{}ns] {}", self.elapsed(), event);
    }
}

#[cfg(feature = "with_diagnostics")]
impl ParThreadLogger for ParLogger {
    fn new(chunk_size: usize) -> Self {
        let logger = Self {
            begin_instant: std::time::Instant::now(),
        };
        logger.log_event(&format!("init with chunk size = {}", chunk_size));
        logger
    }

    fn next_chunk(&self, chunk_size: usize) {
        self.log_event(&format!("pull chunk size = {}", chunk_size));
    }

    fn log_num_spawned(num_spawned: usize) {
        println!("[PAR] num-spawned = {}", num_spawned);
    }
}

#[cfg(feature = "with_diagnostics")]
impl Drop for ParLogger {
    fn drop(&mut self) {
        self.log_event("drop");
    }
}
