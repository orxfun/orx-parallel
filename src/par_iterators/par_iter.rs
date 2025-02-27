use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
    parameters::{ChunkSize, NumThreads},
};

pub trait ParIterCore {
    fn input_len(&self) -> Option<usize>;
}

pub trait ParIter<R = DefaultRunner>: ParIterCore + Sized
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    // transform

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.input_len());
        self.collect_into(output)
    }
}
