use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParallelRunner {
    fn new(params: Params, kind: ComputationKind, iter: &impl ConcurrentIter) -> Self;

    fn run<I, T1, TN>(&self, iter: &I, run_one_by_one: &T1, run_in_chunks: &TN) -> usize
    where
        I: ConcurrentIter,
        T1: Fn() + Sync,
        TN: Fn(usize) + Sync;
}
