mod args;
mod table;

use crate::args::RunnerArgs;
use crate::table::Table;
use clap::Parser;
use indicatif::ProgressBar;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use toml_edit::DocumentMut;

fn main() {
    let args = RunnerArgs::parse();
    assert!(!args.threads.is_empty());

    run_benchmark(
        &args.path,
        &args.path_result,
        args.warmup_runs,
        args.actual_runs,
        &args.threads,
    );
}

pub fn run_benchmark(
    path: &PathBuf,
    path_result: &PathBuf,
    warmup_runs: usize,
    actual_runs: usize,
    threads: &[usize],
) {
    let methods = get_method_features(path);
    let inputs = get_input_factors(&path, warmup_runs, actual_runs, &methods[0]);

    println!(
        "\nmethods={methods:?}\nthreads={:?}\ninputs={inputs:?}\n",
        threads
    );

    let bar = ProgressBar::new((methods.len() * threads.len()) as u64);
    let mut table = Table::new(inputs);

    for method in &methods {
        for &threads in threads {
            sleep(Duration::from_millis(500));
            let output = run_once(path, warmup_runs, actual_runs, threads, method);
            table.append(output, threads);
            bar.inc(1);
        }
    }

    table.print();
    table.write_csv(path_result);
    println!("\nwritten to:\n{}\n", path_result.to_string_lossy());
}

fn command(
    path: &PathBuf,
    warmup_runs: usize,
    actual_runs: usize,
    threads: usize,
    method: &str,
    mode: &str,
) -> Command {
    let mut command = Command::new("cargo");

    command
        .current_dir(path)
        .env("RAYON_NUM_THREADS", threads.to_string())
        .env("ORX_PARALLEL_MAX_NUM_THREADS", threads.to_string());

    let mut cli_args = vec!["run".to_string(), "--release".to_string()];

    cli_args.extend(["--features".to_string(), method.to_string()]);

    cli_args.push("--".to_string());

    cli_args.extend(["--warmup-runs".to_string(), warmup_runs.to_string()]);
    cli_args.extend(["--actual-runs".to_string(), actual_runs.to_string()]);

    cli_args.extend(["--run-mode".to_string(), mode.to_string()]);

    command.args(cli_args);

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    command
}

fn get_method_features(path: &PathBuf) -> Vec<String> {
    let path = path.join("Cargo.toml");
    let content = fs::read_to_string(path).unwrap();

    // Parse with toml_edit (preserves order)
    let doc = content.parse::<DocumentMut>().unwrap();

    // Access the [features] table
    let features = doc["features"]
        .as_table()
        .expect("[features] table missing");

    features.iter().map(|(key, _)| key.to_string()).collect()
}

fn get_input_factors(
    path: &PathBuf,
    warmup_runs: usize,
    actual_runs: usize,
    first_method: &str,
) -> Vec<String> {
    let mut cmd = command(
        path,
        warmup_runs,
        actual_runs,
        1,
        first_method,
        "list-inputs",
    );
    let output = cmd.output().expect("failed to get inputs");
    serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap()
}

fn run_once(
    path: &PathBuf,
    warmup_runs: usize,
    actual_runs: usize,
    threads: usize,
    method: &str,
) -> String {
    let mut cmd = command(path, warmup_runs, actual_runs, threads, method, "run");
    let output = cmd.output().expect("failed to run");
    String::from_utf8_lossy(&output.stdout).to_string()
}
