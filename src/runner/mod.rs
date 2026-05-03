mod par_runner;
mod runner_variants;

pub use par_runner::ParRunner;

// default

pub type DefaultRunner = runner_variants::FixedChunkRunner<crate::pool::StdDefaultPool>;

pub fn default_runner() -> DefaultRunner {
    DefaultRunner::new(Default::default())
}

// recursive

pub type RecursiveRunner = runner_variants::RecursiveChunkRunner<crate::pool::StdDefaultPool>;

pub fn recursive_runner() -> RecursiveRunner {
    RecursiveRunner::new(Default::default())
}
