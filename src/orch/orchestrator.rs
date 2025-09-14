use std::num::NonZeroUsize;

use crate::{
    NumThreads, ParallelRunner, Params,
    orch::{NumSpawned, thread_pool::ParThreadPool},
    runner::ComputationKind,
};
use orx_concurrent_iter::ConcurrentIter;

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
        F: Fn(
                &I,
                &<Self::Runner as ParallelRunner>::SharedState,
                <Self::Runner as ParallelRunner>::ThreadRunner,
            ) + Sync,
    {
        let runner = Self::new_runner(kind, params, iter.try_get_len());
        let state = runner.new_shared_state();
        let do_spawn = |num_spawned| runner.do_spawn_new(num_spawned, &state, &iter);
        let work = || {
            thread_do(&iter, &state, runner.new_thread_runner(&state));
        };
        self.thread_pool_mut().run(do_spawn, work)
    }

    fn map_all<I, M, T, E>(
        &mut self,
        params: Params,
        iter: I,
        kind: ComputationKind,
        thread_map: M,
    ) -> (NumSpawned, Result<Vec<T>, E>)
    where
        I: ConcurrentIter,
        M: Fn(
                &I,
                &<Self::Runner as ParallelRunner>::SharedState,
                <Self::Runner as ParallelRunner>::ThreadRunner,
            ) -> Result<T, E>
            + Sync,
        T: Send,
        E: Send,
    {
        let runner = Self::new_runner(kind, params, iter.try_get_len());
        let state = runner.new_shared_state();
        let do_spawn = |num_spawned| runner.do_spawn_new(num_spawned, &state, &iter);
        let work = || thread_map(&iter, &state, runner.new_thread_runner(&state));
        self.thread_pool_mut().map(do_spawn, work)
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
