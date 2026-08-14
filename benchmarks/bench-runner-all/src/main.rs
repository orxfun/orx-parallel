mod args;

use crate::args::RunnerArgs;
use bench_runner::run_benchmark;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    let args = RunnerArgs::parse();
}

fn command(args: &RunnerArgs, category: &str, bench: &str) {
    let path = args.path.join(category).join(bench);
    let path_result = args.path_result.join(category).join(format!("{bench}.csv"));
}

/*
cargo run --release --
--path ~/orx/orx-parallel/benchmarks/reduce/map
--path-result ~/orx/orx-parallel/docs/bench-ui/results/reduce/map.csv
--warmup-runs 20
--actual-runs 100
--threads 4
--threads 8
--threads 16

*/
