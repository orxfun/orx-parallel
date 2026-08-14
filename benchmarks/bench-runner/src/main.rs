mod args;
mod runner;
mod table;

use crate::args::RunnerArgs;
use clap::Parser;

fn main() {
    let args = RunnerArgs::parse();

    runner::run_benchmark(
        &args.path,
        &args.path_result,
        args.warmup_runs,
        args.actual_runs,
        &args.threads,
    );
}
