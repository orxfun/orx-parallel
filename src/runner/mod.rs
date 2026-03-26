mod computation_kind;
mod par_runner;
mod runner_impl;
mod thread_computations;
mod val_idx;

pub use par_runner::ParRunner;

// default

pub type DefaultRunner = runner_impl::FixedChunkRunner<crate::pool::StdDefaultPool>;

pub fn default_runner() -> DefaultRunner {
    DefaultRunner::new(Default::default())
}
