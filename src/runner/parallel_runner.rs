use super::{
    computation_kind::ComputationKind,
    parallel_task::{ParallelTask, ParallelTaskWithIdx},
    thread_runner::ThreadRunner,
};
use crate::{computations::Values, parameters::Params, runner::parallel_runner_compute};
use orx_concurrent_iter::ConcurrentIter;

/// A parallel runner which is responsible for taking a computation defined as a composition
/// of iterator methods, spawns threads, shares tasks and returns the result of the parallel
/// execution.
pub trait ParallelRunner: Sized + Sync {
    /// Data shared to the thread runners.
    type SharedState: Send + Sync;

    /// Thread runner that is responsible for executing the tasks allocated to a thread.
    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    /// Creates a new parallel runner for the given computation `kind`, parallelization `params`
    /// and `initial_input_len`.
    fn new(kind: ComputationKind, params: Params, initial_input_len: Option<usize>) -> Self;

    /// Creates an initial shared state.
    fn new_shared_state(&self) -> Self::SharedState;

    /// Returns true if it is beneficial to spawn a new thread provided that:
    ///
    /// * `num_spawned` threads are already been spawned, and
    /// * `shared_state` is the current parallel execution state.
    fn do_spawn_new<I>(
        &self,
        num_spawned: usize,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        I: ConcurrentIter;

    /// Creates a new thread runner provided that the current parallel execution state is
    /// `shared_state`.
    fn new_thread_runner(&self, shared_state: &Self::SharedState) -> Self::ThreadRunner;
}

pub trait ParallelRunnerCompute: ParallelRunner {
    // run

    fn run<I, T>(&self, iter: &I, task: T) -> usize
    where
        I: ConcurrentIter,
        T: ParallelTask<Item = I::Item> + Send,
    {
        parallel_runner_compute::run(self, iter, task)
    }

    fn run_with_idx<I, T>(&self, iter: &I, task: T) -> usize
    where
        I: ConcurrentIter,
        T: ParallelTaskWithIdx<Item = I::Item> + Send,
    {
        parallel_runner_compute::run_with_idx(self, iter, task)
    }

    // collect

    #[allow(clippy::type_complexity)]
    fn x_collect_with_idx<I, Vo, M1, CreateM1>(
        &self,
        iter: &I,
        create_map1: CreateM1,
    ) -> (usize, Vec<Vec<(usize, Vo::Item)>>)
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync,
        CreateM1: Fn() -> M1,
    {
        parallel_runner_compute::x_collect_with_idx(self, iter, create_map1)
    }

    #[allow(clippy::type_complexity)]
    fn xfx_collect_with_idx<I, Vt, Vo, M1, F, M2>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> (usize, Vec<Vec<(usize, Vo::Item)>>)
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        parallel_runner_compute::xfx_collect_with_idx(self, iter, map1, filter, map2)
    }

    // reduce

    fn x_reduce<I, Vo, M1, CreateM1, X>(
        &self,
        iter: &I,
        create_map1: CreateM1,
        reduce: &X,
    ) -> (usize, Option<Vo::Item>)
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: FnMut(I::Item) -> Vo + Send + Sync,
        CreateM1: Fn() -> M1,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        parallel_runner_compute::x_reduce(self, iter, create_map1, reduce)
    }

    fn xfx_reduce<I, Vt, Vo, M1, F, M2, X>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
        reduce: &X,
    ) -> (usize, Option<Vo::Item>)
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        parallel_runner_compute::xfx_reduce(self, iter, map1, filter, map2, reduce)
    }

    // next

    fn xfx_next<I, Vt, Vo, M1, F, M2>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> (usize, Option<Vo::Item>)
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        parallel_runner_compute::xfx_next(self, iter, map1, filter, map2)
    }

    fn xfx_next_any<I, Vt, Vo, M1, F, M2>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> (usize, Option<Vo::Item>)
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        parallel_runner_compute::xfx_next_any(self, iter, map1, filter, map2)
    }
}

impl<X: ParallelRunner> ParallelRunnerCompute for X {}
