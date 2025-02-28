use super::par_iter::{ParIter, ParIterCore};
use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
    par_iterators::par_map::ParMap,
    parameters::{ChunkSize, NumThreads, Params},
    IntoPar,
};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
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

impl<I, R> Par<I, R>
where
    I: ConcurrentIter,
    R: ParallelRunner,
{
    pub(crate) fn new(iter: I, params: Params) -> Self {
        Self {
            iter,
            params,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I) {
        (self.params, self.iter)
    }
}

// impl<I, R> IntoConcurrentIter for Par<I, R>
// where
//     I: ConcurrentIter,
//     R: ParallelRunner,
// {
//     type Item = I::Item;

//     type IntoIter = I;

//     fn into_concurrent_iter(self) -> Self::IntoIter {
//         self.iter
//     }
// }

// impl<I, R> IntoPar for Par<I, R>
// where
//     I: ConcurrentIter,
//     R: ParallelRunner,
// {
//     type ParItem = I::Item;

//     type ConIntoIter = I;

//     fn into_par(self) -> Par<Self::ConIntoIter> {
//         self
//     }
// }

impl<I, R> ParIterCore for Par<I, R>
where
    I: ConcurrentIter,
    R: ParallelRunner,
{
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
        let (params, iter) = self.destruct();
        ParMap::new(params, iter, map)
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
