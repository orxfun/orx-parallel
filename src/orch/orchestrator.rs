use crate::{ParallelRunner, Params, runner::ComputationKind};

pub trait Orchestrator {
    type Runner: ParallelRunner;

    fn new_runner(
        &self,
        kind: ComputationKind,
        params: Params,
        initial_input_len: Option<usize>,
    ) -> Self::Runner {
        <Self::Runner as ParallelRunner>::new(kind, params, initial_input_len)
    }
}
