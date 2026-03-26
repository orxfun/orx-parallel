mod computation_kind;
mod par_runner;
mod runner_impl;
mod thread_computations;
mod val_idx;

pub type DefaultRunner = runner_impl::FixedChunkRunner<crate::pool::StdDefaultPool>;

pub use par_runner::ParRunner;
