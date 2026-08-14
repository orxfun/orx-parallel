use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct RunnerArgs {
    /// Path of the directory containing benchmarks
    #[arg(long)]
    pub path: PathBuf,

    /// Path of the folder where results will be written
    #[arg(long)]
    pub path_result: PathBuf,

    /// Number of warmup runs
    #[arg(long)]
    pub warmup_runs: usize,

    /// Number of actual runs to time
    #[arg(long)]
    pub actual_runs: usize,

    /// Number of threads to run experiments with
    #[arg(long)]
    pub threads: Vec<usize>,

    /// Benchmark categories to run
    #[arg(long)]
    pub categories: Vec<String>,
}
