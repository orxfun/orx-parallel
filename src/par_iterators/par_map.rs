use super::par_iter::{ParIter, ParIterCore};
use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
    parameters::{ChunkSize, NumThreads, Params},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParMap<I, O, M, R = DefaultRunner>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    R: ParallelRunner,
{
    params: Params,
    iter: I,
    map: M,
    phantom: PhantomData<R>,
}

impl<I, O, M, R> ParMap<I, O, M, R>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    R: ParallelRunner,
{
    pub(crate) fn new(params: Params, iter: I, map: M) -> Self {
        Self {
            params,
            iter,
            map,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.map)
    }
}

impl<I, O, M, R> ParIterCore for ParMap<I, O, M, R>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    R: ParallelRunner,
{
    fn input_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }
}

impl<I, O, M, R> ParIter<R> for ParMap<I, O, M, R>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    R: ParallelRunner,
{
    type Item = O;

    // params

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    // transform

    fn map<O2, M2>(self, map: M2) -> impl ParIter<Item = O2>
    where
        O2: Send + Sync,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let (params, iter, map1) = self.destruct();
        let map = move |x| map(map1(x));
        ParMap::new(params, iter, map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let (params, iter, map) = self.destruct();
        C::map_into::<_, _, R>(output, params, iter, map)
    }
}
