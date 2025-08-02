use crate::{ParallelRunner, Params, runner::ComputationKind};

pub trait ParallelRunnerCompute: ParallelRunner {
    fn collection(params: Params, len: Option<usize>) -> Self {
        Self::new(ComputationKind::Collect, params, len)
    }

    fn early_return(params: Params, len: Option<usize>) -> Self {
        Self::new(ComputationKind::EarlyReturn, params, len)
    }

    fn reduce(params: Params, len: Option<usize>) -> Self {
        Self::new(ComputationKind::Reduce, params, len)
    }
}

impl<X: ParallelRunner> ParallelRunnerCompute for X {}
