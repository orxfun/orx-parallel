use crate::{
    ParallelRunner, Params,
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

    fn thread_pool(&mut self) -> &Self::ThreadPool;

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

    fn max_num_threads_for_computation(&self, params: Params) -> usize {
        1
    }
}

pub(crate) type SharedStateOf<C> = <<C as Orchestrator>::Runner as ParallelRunner>::SharedState;
pub(crate) type ThreadRunnerOf<C> = <<C as Orchestrator>::Runner as ParallelRunner>::ThreadRunner;
