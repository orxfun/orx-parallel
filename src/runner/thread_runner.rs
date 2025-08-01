use super::parallel_task::{ParallelTask, ParallelTaskWithIdx};
use crate::{computations::Values, runner::thread_runner_compute};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

/// Thread runner responsible for executing the tasks assigned to the thread by the
/// parallel runner.
pub trait ThreadRunner: Sized {
    /// Type of the shared state among threads.
    type SharedState;

    /// Returns the next chunks size to be pulled from the input `iter` for the given
    /// current `shared_state`.
    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> usize
    where
        I: ConcurrentIter;

    /// Hook that will be called before starting to execute the chunk of the given `chunk_size`.
    fn begin_chunk(&mut self, chunk_size: usize);

    /// Hook that will be called after completing the chunk of the given `chunk_size`.
    /// The `shared_state` is also provided so that it can be updated to send information to the
    /// parallel runner and other thread runners.
    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    /// Hook that will be called after completing the task.
    /// The `shared_state` is also provided so that it can be updated to send information to the
    /// parallel runner and other thread runners.
    fn complete_task(&mut self, shared_state: &Self::SharedState);
}

pub(crate) trait ThreadRunnerCompute: ThreadRunner {
    // run

    fn run<I, T>(self, iter: &I, shared_state: &Self::SharedState, task: T)
    where
        I: ConcurrentIter,
        T: ParallelTask<Item = I::Item>,
    {
        thread_runner_compute::run(self, iter, shared_state, task);
    }

    fn run_with_idx<I, T>(self, iter: &I, shared_state: &Self::SharedState, task: T)
    where
        I: ConcurrentIter,
        T: ParallelTaskWithIdx<Item = I::Item>,
    {
        thread_runner_compute::run_with_idx(self, iter, shared_state, task);
    }

    // collect

    fn x_collect_with_idx<I, Vo, M1>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: M1,
    ) -> Vec<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: FnMut(I::Item) -> Vo + Send,
    {
        thread_runner_compute::x_collect_with_idx(self, iter, shared_state, map1)
    }

    fn xfx_collect_with_idx<I, Vt, Vo, M1, F, M2>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Vec<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        thread_runner_compute::xfx_collect_with_idx(self, iter, shared_state, map1, filter, map2)
    }

    // reduce

    fn x_reduce<I, Vo, M1, X>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: M1,
        reduce: &X,
    ) -> Option<Vo::Item>
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: FnMut(I::Item) -> Vo + Send,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        thread_runner_compute::x_reduce(self, iter, shared_state, map1, reduce)
    }

    fn xfx_reduce<I, Vt, Vo, M1, F, M2, X>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
        reduce: &X,
    ) -> Option<Vo::Item>
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
        thread_runner_compute::xfx_reduce(self, iter, shared_state, map1, filter, map2, reduce)
    }

    // next

    fn xfx_next<I, Vt, Vo, M1, F, M2>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Option<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        thread_runner_compute::xfx_next(self, iter, shared_state, map1, filter, map2)
    }

    fn xfx_next_any<I, Vt, Vo, M1, F, M2>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Option<Vo::Item>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        thread_runner_compute::xfx_next_any(self, iter, shared_state, map1, filter, map2)
    }
}

impl<X: ThreadRunner> ThreadRunnerCompute for X {}
