use super::{chunk_size::ChunkSize, collect_ordering::CollectOrdering, num_threads::NumThreads};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    pub num_threads: NumThreads,
    pub chunk_size: ChunkSize,
    pub collect_ordering: CollectOrdering,
}

impl Params {
    pub fn new(
        num_threads: impl Into<NumThreads>,
        chunk_size: impl Into<ChunkSize>,
        collect_ordering: CollectOrdering,
    ) -> Self {
        Self {
            num_threads: num_threads.into(),
            chunk_size: chunk_size.into(),
            collect_ordering,
        }
    }

    pub fn is_sequential(self) -> bool {
        self.num_threads.is_sequential()
    }

    // helpers

    pub(crate) fn with_num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self {
            num_threads: num_threads.into(),
            chunk_size: self.chunk_size,
            collect_ordering: self.collect_ordering,
        }
    }

    pub(crate) fn with_chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: chunk_size.into(),
            collect_ordering: self.collect_ordering,
        }
    }

    pub(crate) fn with_collect_ordering(self, collect_ordering: CollectOrdering) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: self.chunk_size,
            collect_ordering,
        }
    }
}
