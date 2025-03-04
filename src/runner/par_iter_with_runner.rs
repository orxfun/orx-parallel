use super::ParallelRunner;
use crate::{ChunkSize, NumThreads, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParIterWithRunner<P, R>
where
    P: ParIter,
    R: ParallelRunner,
{
    iter: P,
    phantom: PhantomData<R>,
}

impl<P, R> ParIter for ParIterWithRunner<P, R>
where
    P: ParIter,
    R: ParallelRunner,
{
    type Item = P::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.iter.con_iter()
    }

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self {
            iter: self.iter.num_threads(num_threads),
            phantom: PhantomData,
        }
    }

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self {
            iter: self.iter.chunk_size(chunk_size),
            phantom: PhantomData,
        }
    }

    fn map<Out, Map>(self, map: Map) -> impl ParIter<Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        ParIterWithRunner::<_, R> {
            iter: self.iter.map(map),
            phantom: PhantomData,
        }
    }

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        todo!()
    }
}
