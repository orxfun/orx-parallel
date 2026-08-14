use clap::{Parser, ValueEnum};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RunMode {
    ListInputs,
    ListMethods,
    Run,
}

impl Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Parser)]
pub struct BenchArgs {
    /// Run mode
    #[arg(long, default_value_t = RunMode::Run)]
    pub run_mode: RunMode,

    /// Number of warmup runs
    #[arg(long)]
    pub warmup_runs: usize,

    /// Number of actual runs to time
    #[arg(long)]
    pub actual_runs: usize,
}
