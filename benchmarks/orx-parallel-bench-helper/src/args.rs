use clap::Parser;

#[derive(Parser)]
pub struct BenchArgs {
    /// When set to true, the program will only return the list of inputs and exit
    #[arg(long, default_value_t = false)]
    pub list_inputs: bool,

    /// Number of warmup runs
    #[arg(long)]
    pub warmup_runs: usize,

    /// Number of actual runs to time
    #[arg(long)]
    pub actual_runs: usize,
}
