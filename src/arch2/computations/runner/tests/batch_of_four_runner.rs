use crate::{
    computations::{
        computation_kind::ComputationKind, runner::thread_runner::ThreadRunner, ParallelRunner,
    },
    parameters::{NumThreads, Params},
};
use orx_concurrent_iter::{ConcurrentIter, Enumeration};

pub struct BatchOfFourParallelRunner(Params);

pub struct BatchOfFourThreadRunner;

impl ThreadRunner for BatchOfFourThreadRunner {
    type SharedState = ();

    fn next_chunk_size<E, I>(&self, _: &Self::SharedState, iter: &I) -> Option<usize>
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
    {
        let x = match iter.try_get_len() {
            Some(0) => None,
            Some(x) if x < 100 => None,
            _ => Some(4),
        };
        dbg!(iter.try_get_len(), &x);
        x
    }

    fn begin_chunk(&mut self, _: usize) {}

    fn complete_chunk(&mut self, _: &Self::SharedState, _: usize) {}

    fn complete_task(&mut self, _: &Self::SharedState) {}
}

impl ParallelRunner for BatchOfFourParallelRunner {
    type SharedState = ();

    type ThreadRunner = BatchOfFourThreadRunner;

    fn new(_: ComputationKind, params: Params, _: Option<usize>) -> Self {
        Self(params)
    }

    fn new_shared_state(&self) -> Self::SharedState {}

    fn do_spawn_new<E, I>(&self, num_spawned: usize, _: &Self::SharedState, iter: &I) -> bool
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
    {
        let max_num_threads = match self.0.num_threads {
            NumThreads::Auto => 8,
            NumThreads::Max(x) => x.into(),
        };

        match (num_spawned, iter.try_get_len()) {
            (_, Some(0)) => false,
            (x, _) if x >= max_num_threads => false,
            _ => true,
        }
    }

    fn new_thread_runner(&self, _: &Self::SharedState) -> Self::ThreadRunner {
        BatchOfFourThreadRunner
    }
}
