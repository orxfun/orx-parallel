mod new_runner;
mod par_runner;
mod runner_variants;

pub use new_runner::Runner;
pub use par_runner::ParRunner;
#[cfg(feature = "std")]
pub use runner_variants::enable_runner_b_diagnostics;
#[cfg(feature = "std")]
pub use runner_variants::take_last_runner_b_diagnostics;
#[cfg(feature = "std")]
pub use runner_variants::DiagnosticData;

// default

pub type DefaultRunner = runner_variants::FixedChunkRunner<crate::pool::DefaultPool>;

pub fn default_runner() -> DefaultRunner {
    DefaultRunner::new(Default::default())
}
