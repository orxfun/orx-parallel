use crate::computations::Values;
use crate::{ChunkSize, CollectOrdering, NumThreads, Params};
use orx_concurrent_iter::ConcurrentIter;

pub struct Mfm<I, T, Vt, O, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values<Item = T>,
    O: Send + Sync,
    Vo: Values<Item = O>,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    params: Params,
    iter: I,
    map1: M1,
    filter: F,
    map2: M2,
}

impl<I, T, Vt, O, Vo, M1, F, M2> Mfm<I, T, Vt, O, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values<Item = T>,
    O: Send + Sync,
    Vo: Values<Item = O>,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    pub fn new(params: Params, iter: I, map1: M1, filter: F, map2: M2) -> Self {
        Self {
            params,
            iter,
            map1,
            filter,
            map2,
        }
    }

    pub fn destruct(self) -> (Params, I, M1, F, M2) {
        (self.params, self.iter, self.map1, self.filter, self.map2)
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn num_threads(&mut self, num_threads: impl Into<NumThreads>) {
        self.params = self.params().with_num_threads(num_threads);
    }

    pub fn chunk_size(&mut self, chunk_size: impl Into<ChunkSize>) {
        self.params = self.params.with_chunk_size(chunk_size);
    }

    pub fn collect_ordering(&mut self, collect: CollectOrdering) {
        self.params = self.params.with_collect_ordering(collect);
    }

    pub fn iter(&self) -> &I {
        &self.iter
    }

    pub fn par_len(&self) -> Option<usize> {
        match (self.params.is_sequential(), self.iter.try_get_len()) {
            (true, _) => None, // not required to concurrent reserve when seq
            (false, x) => x,
        }
    }
}
