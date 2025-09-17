use crate::{
    NumThreads, ParallelExecutor, Params,
    generic_values::runner_results::{Fallibility, Infallible, Never},
    par_thread_pool::{ParThreadPool, ParThreadPoolCompute},
    runner::{ComputationKind, NumSpawned},
};
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use orx_concurrent_iter::ConcurrentIter;

pub trait Orchestrator {
    type Runner: ParallelExecutor;

    type ThreadPool: ParThreadPool;

    fn new_runner(
        &self,
        kind: ComputationKind,
        params: Params,
        initial_input_len: Option<usize>,
    ) -> Self::Runner {
        let max_num_threads = self.max_num_threads_for_computation(params, initial_input_len);
        <Self::Runner as ParallelExecutor>::new(kind, params, initial_input_len, max_num_threads)
    }

    fn thread_pool(&self) -> &Self::ThreadPool;

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool;

    // derived

    fn run_all<I, F>(
        &mut self,
        params: Params,
        iter: I,
        kind: ComputationKind,
        thread_do: F,
    ) -> NumSpawned
    where
        I: ConcurrentIter,
        F: Fn(NumSpawned, &I, &SharedStateOf<Self>, ThreadRunnerOf<Self>) + Sync,
    {
        let runner = self.new_runner(kind, params, iter.try_get_len());
        let state = runner.new_shared_state();
        let do_spawn = |num_spawned| runner.do_spawn_new(num_spawned, &state, &iter);
        let work = |num_spawned| {
            thread_do(
                num_spawned,
                &iter,
                &state,
                runner.new_thread_executor(&state),
            );
        };
        self.thread_pool_mut().run_in_pool(do_spawn, work)
    }

    fn map_all<F, I, M, T>(
        &mut self,
        params: Params,
        iter: I,
        kind: ComputationKind,
        thread_map: M,
    ) -> (NumSpawned, Result<Vec<T>, F::Error>)
    where
        F: Fallibility,
        I: ConcurrentIter,
        M: Fn(NumSpawned, &I, &SharedStateOf<Self>, ThreadRunnerOf<Self>) -> Result<T, F::Error>
            + Sync,
        T: Send,
        F::Error: Send,
    {
        let iter_len = iter.try_get_len();
        let runner = self.new_runner(kind, params, iter_len);
        let state = runner.new_shared_state();
        let do_spawn = |num_spawned| runner.do_spawn_new(num_spawned, &state, &iter);
        let work = |nt| thread_map(nt, &iter, &state, runner.new_thread_executor(&state));
        let max_num_threads = self.max_num_threads_for_computation(params, iter_len);
        self.thread_pool_mut()
            .map_in_pool::<F, _, _, _>(do_spawn, work, max_num_threads)
    }

    fn map_infallible<I, M, T>(
        &mut self,
        params: Params,
        iter: I,
        kind: ComputationKind,
        thread_map: M,
    ) -> (NumSpawned, Result<Vec<T>, Never>)
    where
        I: ConcurrentIter,
        M: Fn(NumSpawned, &I, &SharedStateOf<Self>, ThreadRunnerOf<Self>) -> Result<T, Never>
            + Sync,
        T: Send,
    {
        self.map_all::<Infallible, _, _, _>(params, iter, kind, thread_map)
    }

    fn max_num_threads_for_computation(
        &self,
        params: Params,
        iter_len: Option<usize>,
    ) -> NonZeroUsize {
        let pool = self.thread_pool().max_num_threads();

        let env = crate::env::max_num_threads_by_env_variable().unwrap_or(NonZeroUsize::MAX);

        let req = match (iter_len, params.num_threads) {
            (Some(len), NumThreads::Auto) => NonZeroUsize::new(len.max(1)).expect(">0"),
            (Some(len), NumThreads::Max(nt)) => NonZeroUsize::new(len.max(1)).expect(">0").min(nt),
            (None, NumThreads::Auto) => NonZeroUsize::MAX,
            (None, NumThreads::Max(nt)) => nt,
        };

        req.min(pool.min(env))
    }
}

pub(crate) type SharedStateOf<C> = <<C as Orchestrator>::Runner as ParallelExecutor>::SharedState;
pub(crate) type ThreadRunnerOf<C> =
    <<C as Orchestrator>::Runner as ParallelExecutor>::ThreadExecutor;

// auto impl for &mut pool

impl<'a, O> Orchestrator for &'a mut O
where
    O: Orchestrator,
{
    type Runner = O::Runner;

    type ThreadPool = O::ThreadPool;

    fn thread_pool(&self) -> &Self::ThreadPool {
        O::thread_pool(self)
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        O::thread_pool_mut(self)
    }
}
