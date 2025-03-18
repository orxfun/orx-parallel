use super::{chunk_size::ChunkSize, collect::Collect, num_threads::NumThreads};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    pub num_threads: NumThreads,
    pub chunk_size: ChunkSize,
    pub collect: Collect,
}

impl Params {
    pub fn is_sequential(self) -> bool {
        self.num_threads.is_sequential()
    }

    // helpers

    pub(crate) fn with_num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self {
            num_threads: num_threads.into(),
            chunk_size: self.chunk_size,
            collect: self.collect,
        }
    }

    pub(crate) fn with_chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: chunk_size.into(),
            collect: self.collect,
        }
    }

    pub(crate) fn with_collect_approach(self, collect: Collect) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: self.chunk_size,
            collect,
        }
    }
}
