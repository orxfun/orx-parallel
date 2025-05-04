use super::{chunk_size::ChunkSize, iteration_order::IterationOrder, num_threads::NumThreads};

/// Parameters of a parallel computation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    /// Number of threads to be used in the parallel computation.
    ///
    /// See [`NumThreads`] for details.
    pub num_threads: NumThreads,
    /// Chunk size to be used in the parallel computation.
    ///
    /// See [`ChunkSize`] for details.
    pub chunk_size: ChunkSize,
    /// Ordering of outputs of the parallel computation that is important when the outputs
    /// are collected into a collection.
    ///
    /// See [`CollectOrdering`] for details.
    pub collect_ordering: IterationOrder,
}

impl Params {
    /// Crates parallel computation parameters for the given configurations.
    pub fn new(
        num_threads: impl Into<NumThreads>,
        chunk_size: impl Into<ChunkSize>,
        collect_ordering: IterationOrder,
    ) -> Self {
        Self {
            num_threads: num_threads.into(),
            chunk_size: chunk_size.into(),
            collect_ordering,
        }
    }

    /// Returns true if number of threads is set to 1.
    ///
    /// Note that in this case the computation will be executed sequentially using regular iterators.
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

    pub(crate) fn with_collect_ordering(self, collect_ordering: IterationOrder) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: self.chunk_size,
            collect_ordering,
        }
    }
}
