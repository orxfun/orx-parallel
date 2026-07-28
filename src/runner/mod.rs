mod new_runner;
mod par_runner;
mod runner_variants;

pub use new_runner::Runner;
pub use par_runner::ParRunner;

// default

#[cfg(not(feature = "std"))]
pub type DefaultRunner = runner_variants::FixedChunkRunner<crate::pool::DefaultPool>;

#[cfg(feature = "std")]
pub type DefaultRunner = runner_variants::AdaptiveChunkRunner<crate::pool::DefaultPool>;

pub fn default_runner() -> DefaultRunner {
    DefaultRunner::new(Default::default())
}
