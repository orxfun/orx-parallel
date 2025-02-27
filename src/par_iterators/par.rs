use super::par_iter::{ParIter, ParIterCore};
use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
    par_iterators::par_map::ParMap,
    parameters::{ChunkSize, NumThreads, Params},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct Par<I, R = DefaultRunner>
where
    I: ConcurrentIter,
    R: ParallelRunner,
{
    iter: I,
    params: Params,
    phantom: PhantomData<R>,
}

impl<I: ConcurrentIter> Par<I> {
    pub(crate) fn new(iter: I, params: Params) -> Self {
        Self {
            iter,
            params,
            phantom: PhantomData,
        }
    }
}

impl<I: ConcurrentIter, R: ParallelRunner> ParIterCore for Par<I, R> {
    fn input_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }
}

impl<I, R> ParIter<R> for Par<I, R>
where
    I: ConcurrentIter,
    R: ParallelRunner,
{
    type Item = I::Item;

    // transform

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let map = ParMap::new(self.params, self.iter, no_ops_map);
        ParIter::<R>::collect_into(map, output)
    }
}

#[inline(always)]
fn no_ops_map<T>(input: T) -> T {
    input
}
