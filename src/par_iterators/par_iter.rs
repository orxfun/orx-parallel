use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
    parameters::{ChunkSize, NumThreads},
    IntoPar,
};

pub trait ParIterCore {
    fn input_len(&self) -> Option<usize>;
}

pub trait ParIter<R = DefaultRunner>:
    ParIterCore + Sized + IntoPar<R, ParItem = Self::Item>
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    // transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q>;

    // transform

    fn map<O2, M2>(self, map: M2) -> impl ParIter<Item = O2>
    where
        O2: Send + Sync,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone;

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
