use crate::{ChunkSize, IterationOrder, NumThreads, Params, computations::Values};
use orx_concurrent_iter::ConcurrentIter;

pub struct UsingX<U, I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U, I::Item) -> Vo + Send + Sync,
{
    params: Params,
    using: U,
    iter: I,
    xap1: M1,
}

impl<U, I, Vo, M1> UsingX<U, I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U, I::Item) -> Vo + Send + Sync,
{
    pub fn new(params: Params, using: U, iter: I, xap1: M1) -> Self {
        Self {
            params,
            using,
            iter,
            xap1,
        }
    }

    pub fn destruct(self) -> (Params, U, I, M1) {
        (self.params, self.using, self.iter, self.xap1)
    }

    pub fn params(&self) -> Params {
        self.params
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
