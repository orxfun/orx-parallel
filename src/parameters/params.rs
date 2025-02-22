use super::{chunk_size::ChunkSize, num_threads::NumThreads};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    pub num_threads: NumThreads,
    pub chunk_size: ChunkSize,
}

impl Params {
    pub fn is_sequential(self) -> bool {
        self.num_threads.is_sequential()
    }
}
