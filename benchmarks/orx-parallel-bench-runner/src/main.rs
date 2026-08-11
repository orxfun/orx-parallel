mod args;
mod table;

use crate::args::RunnerArgs;
use crate::table::Table;
use clap::Parser;
use std::fs;
use std::process::{Command, Stdio};
use toml_edit::DocumentMut;

fn main() {
    let args = RunnerArgs::parse();
    assert!(!args.threads.is_empty());
    println!("{args:?}");

    let methods = get_method_features(&args);
    let inputs = get_input_factors(&args, &methods[0]);
    println!("{methods:?}");
    println!("{inputs:?}");

    let mut table = Table::new(inputs);

    for method in &methods {
        for &threads in &args.threads {
            println!("\n\n\n{method} => {threads}");
            let output = run_once(&args, threads, method);
            table.append(output, threads);
        }
    }

    println!("\n\n\n{table:?}");
    table.write_csv(&args.path_result);
}

fn command(args: &RunnerArgs, threads: usize, method: &str, mode: &str) -> Command {
    let mut command = Command::new("cargo");

    command
        .current_dir(&args.path)
        .env("RAYON_NUM_THREADS", threads.to_string())
        .env("ORX_PARALLEL_MAX_NUM_THREADS", threads.to_string());

    let mut cli_args = vec!["run".to_string(), "--release".to_string()];

    cli_args.extend(["--features".to_string(), method.to_string()]);

    cli_args.push("--".to_string());

    cli_args.extend(["--warmup-runs".to_string(), args.warmup_runs.to_string()]);
    cli_args.extend(["--actual-runs".to_string(), args.actual_runs.to_string()]);

    cli_args.extend(["--run-mode".to_string(), mode.to_string()]);

    command.args(cli_args);

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    command
}

fn get_method_features(args: &RunnerArgs) -> Vec<String> {
    let path = format!("{}/Cargo.toml", args.path);
    let content = fs::read_to_string(path).unwrap();

    // Parse with toml_edit (preserves order)
    let doc = content.parse::<DocumentMut>().unwrap();

    // Access the [features] table
    let features = doc["features"]
        .as_table()
        .expect("[features] table missing");

    features.iter().map(|(key, _)| key.to_string()).collect()
}

fn get_input_factors(args: &RunnerArgs, first_method: &str) -> Vec<String> {
    let mut cmd = command(&args, 1, first_method, "list-inputs");
    let output = cmd.output().expect("failed to get inputs");
    serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap()
}

fn run_once(args: &RunnerArgs, threads: usize, method: &str) -> String {
    let mut cmd = command(args, threads, method, "run");
    let output = cmd.output().expect("failed to run");
    String::from_utf8_lossy(&output.stdout).to_string()
}
