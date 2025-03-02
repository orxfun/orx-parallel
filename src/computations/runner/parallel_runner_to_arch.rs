use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, Enumeration};

pub trait ParallelRunnerToArchive {
    fn new(kind: ComputationKind, params: Params, initial_len: Option<usize>) -> Self;

    fn run<I, E, R>(&self, iter: &I, run: &R) -> usize
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
        R: Fn(usize) + Sync;
}
