mod args;

use crate::args::RunnerArgs;
use bench_runner::run_benchmark;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    let args = RunnerArgs::parse();

    let dir_bench_categories: Vec<_> = args.categories.iter().map(|x| args.path.join(x)).collect();
    dir_bench_categories
        .iter()
        .for_each(|x| assert!(x.exists()));

    let dir_result_categories: Vec<_> = args
        .categories
        .iter()
        .map(|x| args.path_result.join(x))
        .collect();
    dir_result_categories
        .iter()
        .for_each(|x| std::fs::create_dir_all(x).unwrap());

    for category in &args.categories {
        run_for_category(&args, category);
    }
}

fn run_for_category(args: &RunnerArgs, category: &str) {
    let dir_bench_category = args.path.join(category);
    let dir_result_category = args.path_result.join(category);
    let RunnerArgs {
        warmup_runs,
        actual_runs,
        threads,
        ..
    } = args;

    for path in list_dirs(&dir_bench_category).unwrap() {
        let bench = path.file_name().unwrap().to_str().unwrap();
        let path_result = dir_result_category.join(&format!("{bench}.csv"));

        println!("\n\n# {category}/{bench}");
        run_benchmark(&path, &path_result, *warmup_runs, *actual_runs, threads);
    }
}

fn list_dirs(path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            dirs.push(entry.path());
        }
    }

    Ok(dirs)
}
