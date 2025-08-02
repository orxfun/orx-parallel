use crate::computations::Values;
use crate::computations::using::Using;
use crate::{ChunkSize, IterationOrder, NumThreads, Params};
use orx_concurrent_iter::ConcurrentIter;

pub struct Xfx<U, I, Vt, Vo, M1, F, M2>
where
    U: Using,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
{
    using: U,
    params: Params,
    iter: I,
    xap1: M1,
    filter: F,
    xap2: M2,
}

impl<U, I, Vt, Vo, M1, F, M2> Xfx<U, I, Vt, Vo, M1, F, M2>
where
    U: Using,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
{
    pub fn new(using: U, params: Params, iter: I, xap1: M1, filter: F, xap2: M2) -> Self {
        Self {
            using,
            params,
            iter,
            xap1,
            filter,
            xap2,
        }
    }

    pub fn destruct(self) -> (U, Params, I, M1, F, M2) {
        (
            self.using,
            self.params,
            self.iter,
            self.xap1,
            self.filter,
            self.xap2,
        )
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
