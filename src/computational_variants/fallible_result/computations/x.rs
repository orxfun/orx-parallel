use crate::{
    ChunkSize, IterationOrder, NumThreads, Params, generic_values::Values, orch::Orchestrator,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct X<R, I, Vo, X1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(I::Item) -> Vo,
{
    orchestrator: R,
    params: Params,
    iter: I,
    xap1: X1,
}

impl<R, I, Vo, M1> X<R, I, Vo, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(I::Item) -> Vo,
{
    pub fn new(orchestrator: R, params: Params, iter: I, xap1: M1) -> Self {
        Self {
            orchestrator,
            params,
            iter,
            xap1,
        }
    }

    pub fn destruct(self) -> (R, Params, I, M1) {
        (self.orchestrator, self.params, self.iter, self.xap1)
    }

    pub fn params(&self) -> Params {
        self.params
    }

    pub fn len_and_params(&self) -> (Option<usize>, Params) {
        (self.iter.try_get_len(), self.params)
    }

    pub fn num_threads(&mut self, num_threads: impl Into<NumThreads>) {
        self.params = self.params.with_num_threads(num_threads);
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
