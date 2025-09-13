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

    fn thread_pool(&mut self) -> &mut Self::ThreadPool;

    // derived

    fn run<I, F>(
        &mut self,
        params: Params,
        iter: I,
        kind: ComputationKind,
        thread_work: F,
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
            thread_work(&iter, &state, runner.new_thread_runner(&state));
        };
        self.thread_pool().run(do_spawn, work)
    }
}

pub(crate) type SharedStateOf<C> = <<C as Orchestrator>::Runner as ParallelRunner>::SharedState;
pub(crate) type ThreadRunnerOf<C> = <<C as Orchestrator>::Runner as ParallelRunner>::ThreadRunner;
