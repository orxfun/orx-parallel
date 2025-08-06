use crate::{ChunkSize, IterationOrder, NumThreads, Params, computations::Values};
use orx_concurrent_iter::ConcurrentIter;

pub struct X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(I::Item) -> Vo + Sync,
{
    params: Params,
    iter: I,
    xap1: M1,
}

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub fn new(params: Params, iter: I, xap1: M1) -> Self {
        Self { params, iter, xap1 }
    }

    pub fn destruct(self) -> (Params, I, M1) {
        (self.params, self.iter, self.xap1)
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
