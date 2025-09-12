use crate::{ParallelRunner, Params, orch::thread_pool::ParThreadPool, runner::ComputationKind};

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
}
