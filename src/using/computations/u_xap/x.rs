use crate::using::Using;
use crate::{ChunkSize, IterationOrder, NumThreads, Params, generic_values::Values};
use orx_concurrent_iter::ConcurrentIter;

pub struct UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(&mut U::Item, I::Item) -> Vo,
{
    using: U,
    params: Params,
    iter: I,
    xap1: M1,
}

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(&mut U::Item, I::Item) -> Vo,
{
    pub fn new(using: U, params: Params, iter: I, xap1: M1) -> Self {
        Self {
            using,
            params,
            iter,
            xap1,
        }
    }

    pub fn destruct(self) -> (U, Params, I, M1) {
        (self.using, self.params, self.iter, self.xap1)
    }

    pub fn params(&self) -> Params {
        self.params
    }

    pub fn len_and_params(&self) -> (Option<usize>, Params) {
        (self.iter.try_get_len(), self.params)
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

    pub(crate) fn iter(&self) -> &I {
        &self.iter
    }
}
