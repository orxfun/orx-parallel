use crate::parameters::{ChunkSize, NumThreads};

pub trait ParIter: Sized {
    type Item: Send + Sync;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;
}
