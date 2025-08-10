use crate::{ChunkSize, IterationOrder, NumThreads, Params, using::Using};
use orx_concurrent_iter::ConcurrentIter;

pub struct UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O,
{
    using: U,
    params: Params,
    iter: I,
    map1: M1,
}

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O,
{
    pub fn new(using: U, params: Params, iter: I, map1: M1) -> Self {
        Self {
            using,
            params,
            iter,
            map1,
        }
    }

    pub fn destruct(self) -> (U, Params, I, M1) {
        (self.using, self.params, self.iter, self.map1)
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
