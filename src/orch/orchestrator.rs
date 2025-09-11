use super::par_scope::ParScope;
use crate::{ParallelRunner, Params, runner::ComputationKind};

pub trait Orchestrator {
    type Runner: ParallelRunner;

    type Scope<'env, 'scope>: ParScope<'env, 'scope>
    where
        'env: 'scope;

    fn new_runner(
        &self,
        kind: ComputationKind,
        params: Params,
        initial_input_len: Option<usize>,
    ) -> Self::Runner {
        <Self::Runner as ParallelRunner>::new(kind, params, initial_input_len)
    }

    fn scope<'env, F, T>(f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope Self::Scope<'env, 'scope>) -> T;
}
