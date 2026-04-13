mod par_runner;
mod runner_variants;

pub use par_runner::ParRunner;
pub use runner_variants::RunnerWithDiagnostics;

// default

pub type DefaultRunner = runner_variants::FixedChunkRunner<crate::pool::StdDefaultPool>;

pub fn default_runner() -> DefaultRunner {
    DefaultRunner::new(Default::default())
}
