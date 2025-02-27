use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParallelRunner {
    fn new(params: Params, kind: ComputationKind, iter: &impl ConcurrentIter) -> Self;

    fn run<I, R>(&self, iter: &I, run: &R) -> usize
    where
        I: ConcurrentIter,
        R: Fn(usize) + Sync;
}
