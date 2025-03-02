use super::thread_runner::MyThreadRunner;
use crate::{
    computations::{computation_kind::ComputationKind, runner::parallel_runner::ParallelRunner},
    parameters::Params,
};
use orx_concurrent_iter::{ConcurrentIter, Enumeration};
use std::sync::atomic::AtomicUsize;

pub struct MyParallelRunner;

impl ParallelRunner for MyParallelRunner {
    type SharedState = AtomicUsize;

    type ThreadRunner = MyThreadRunner;

    fn new_shared_state<E, I>(kind: ComputationKind, params: Params, iter: &I) -> Self::SharedState
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
    {
        0.into()
    }

    fn do_spawn_new<E, I>(num_spawned: usize, shared_state: &Self::SharedState, iter: &I) -> bool
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
    {
        todo!()
    }
}
