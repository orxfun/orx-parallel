use crate::{ChunkSize, IterationOrder, NumThreads, Params};
use orx_concurrent_iter::ConcurrentIter;

pub struct M<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
{
    params: Params,
    iter: I,
    cmv: T,
    map1: M1,
}

impl<I, T, O, M1> M<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
{
    pub fn new(params: Params, iter: I, cmv: T, map1: M1) -> Self {
        Self {
            params,
            iter,
            cmv,
            map1,
        }
    }

    pub fn destruct(self) -> (Params, I, T, M1) {
        (self.params, self.iter, self.cmv, self.map1)
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

    pub fn iteration_order(&mut self, collect: IterationOrder) {
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
