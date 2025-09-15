use crate::{
    NumThreads, ParallelRunner, Params,
    generic_values::runner_results::{Fallibility, Infallible, Never},
    orch::{
        NumSpawned,
        thread_pool::{ParThreadPool, ParThreadPoolCompute},
    },
    runner::ComputationKind,
};
use orx_concurrent_iter::ConcurrentIter;
use std::num::NonZeroUsize;

pub trait Orchestrator {
    type Runner: ParallelRunner;

    type ThreadPool: ParThreadPool;

    fn new_runner(
        kind: ComputationKind,
        params: Params,
        initial_input_len: Option<usize>,
    ) -> Self::Runner {
        <Self::Runner as ParallelRunner>::new(kind, params, initial_input_len)
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
        F: Fn(&I, &SharedStateOf<Self>, ThreadRunnerOf<Self>) + Sync,
    {
        let runner = Self::new_runner(kind, params, iter.try_get_len());
        let state = runner.new_shared_state();
        let do_spawn = |num_spawned| runner.do_spawn_new(num_spawned, &state, &iter);
        let work = || {
            thread_do(&iter, &state, runner.new_thread_runner(&state));
        };
        self.thread_pool_mut().run(do_spawn, work)
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
        M: Fn(&I, &SharedStateOf<Self>, ThreadRunnerOf<Self>) -> Result<T, F::Error> + Sync,
        T: Send,
        F::Error: Send,
    {
        let iter_len = iter.try_get_len();
        let runner = Self::new_runner(kind, params, iter_len);
        let state = runner.new_shared_state();
        let do_spawn = |num_spawned| runner.do_spawn_new(num_spawned, &state, &iter);
        let work = || thread_map(&iter, &state, runner.new_thread_runner(&state));
        let max_num_threads = self.max_num_threads_for_computation(params, iter_len);
        self.thread_pool_mut()
            .map_all::<F, _, _, _>(do_spawn, work, max_num_threads)
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
        M: Fn(&I, &SharedStateOf<Self>, ThreadRunnerOf<Self>) -> Result<T, Never> + Sync,
        T: Send,
    {
        self.map_all::<Infallible, _, _, _>(params, iter, kind, thread_map)
    }

    fn max_num_threads_for_computation(
        &self,
        params: Params,
        iter_len: Option<usize>,
    ) -> NonZeroUsize {
        let ava = self.thread_pool().max_num_threads();

        let req = match (iter_len, params.num_threads) {
            (Some(len), NumThreads::Auto) => NonZeroUsize::new(len.max(1)).expect(">0"),
            (Some(len), NumThreads::Max(nt)) => NonZeroUsize::new(len.max(1)).expect(">0").min(nt),
            (None, NumThreads::Auto) => NonZeroUsize::MAX,
            (None, NumThreads::Max(nt)) => nt,
        };

        req.min(ava)
    }
}

pub(crate) type SharedStateOf<C> = <<C as Orchestrator>::Runner as ParallelRunner>::SharedState;
pub(crate) type ThreadRunnerOf<C> = <<C as Orchestrator>::Runner as ParallelRunner>::ThreadRunner;

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
