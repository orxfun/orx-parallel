use clap::Parser;

#[derive(Parser, Debug)]
pub struct RunnerArgs {
    /// When set to true, the program will only return the list of inputs and exit
    #[arg(long)]
    pub path: String,

    /// Path of the csv file where results will be written
    #[arg(long)]
    pub path_result: String,

    /// Number of warmup runs
    #[arg(long)]
    pub warmup_runs: usize,

    /// Number of actual runs to time
    #[arg(long)]
    pub actual_runs: usize,

    /// Number of threads to run experiments with
    #[arg(long)]
    pub threads: Vec<usize>,
}
