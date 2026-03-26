mod computation_kind;
mod par_runner;
mod runner_impl;
mod thread_computations;
mod val_idx;

pub type DEFAULT_RUNNER = runner_impl::FixedChunkRunner<crate::pool::StdDefaultPool>;
