mod alloc_tracking;
mod locations;
mod par_with_use;
mod par_without_use;
mod rand_utils;

use crate::par_with_use::run_search_parallel_use_mut;
use crate::par_without_use::run_search_parallel_immutable;
use crate::{alloc_tracking::AllocationStats, locations::locations};
use clap::Parser;
use std::hint::black_box;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "use_demo_tsp")]
#[command(about = "Compare immutable vs use_vec TSP search")]
struct Args {
    #[arg(long, default_value_t = 100)]
    iterations: usize,

    #[arg(long, default_value_t = 4)]
    threads: usize,

    #[arg(long, default_value_t = 50)]
    num_cities: usize,
}

fn average_duration_and_allocs<F, T>(rounds: u32, mut run: F) -> (Duration, AllocationStats, T)
where
    F: FnMut() -> T,
{
    let mut total = Duration::ZERO;
    let mut total_allocs = AllocationStats::default();
    let mut last_result = None;

    for _ in 0..rounds {
        AllocationStats::reset();
        let start = Instant::now();
        let result = run();
        total += start.elapsed();
        total_allocs = total_allocs.add(AllocationStats::read());
        last_result = Some(result);
    }

    (
        total / rounds,
        total_allocs.div(rounds as u64),
        last_result.expect("rounds must be > 0"),
    )
}

fn ratio(use_vec_value: f64, immutable_value: f64) -> f64 {
    if immutable_value == 0.0 {
        f64::NAN
    } else {
        use_vec_value / immutable_value
    }
}

fn main() {
    let args = Args::parse();
    let iterations = args.iterations;
    let threads = args.threads;
    let num_cities = args.num_cities;
    let seed = 42;
    let rounds = 5;

    let locations: Vec<_> = locations(num_cities);

    // Warm up both paths to avoid one-time initialization costs skewing results.
    black_box(run_search_parallel_immutable(
        &locations, iterations, seed, threads,
    ));
    black_box(run_search_parallel_use_mut(
        &locations, iterations, seed, threads,
    ));

    let (immutable_avg, immutable_allocs, immutable_result) =
        average_duration_and_allocs(rounds, || {
            run_search_parallel_immutable(&locations, iterations, seed, threads)
        });
    let (use_vec_avg, use_vec_allocs, use_vec_result) = average_duration_and_allocs(rounds, || {
        run_search_parallel_use_mut(&locations, iterations, seed, threads)
    });

    let immutable_best = immutable_result
        .as_ref()
        .map(|(_, distance)| *distance)
        .unwrap_or(f64::NAN);
    let use_vec_best = use_vec_result
        .as_ref()
        .map(|(_, distance)| *distance)
        .unwrap_or(f64::NAN);

    println!(
        "iterations: {iterations}, threads: {threads}, cities: {num_cities}, rounds: {rounds}"
    );
    println!();
    println!(
        "| {:<10} | {:<12} | {:<20} | {:<21} |",
        "method", "avg time", "avg allocation calls", "avg allocation bytes"
    );
    println!("| {:-<10} | {:-<12} | {:-<20} | {:-<21} |", "", "", "", "");
    println!(
        "| {:<10} | {:>9.3} ms | {:>20} | {:>19} b |",
        "immutable",
        immutable_avg.as_secs_f64() * 1e3,
        immutable_allocs.alloc_calls,
        immutable_allocs.gross_allocated_bytes(),
    );
    println!(
        "| {:<10} | {:>9.3} ms | {:>20} | {:>19} b |",
        "use_vec",
        use_vec_avg.as_secs_f64() * 1e3,
        use_vec_allocs.alloc_calls,
        use_vec_allocs.gross_allocated_bytes(),
    );

    let time_ratio = ratio(use_vec_avg.as_secs_f64(), immutable_avg.as_secs_f64());
    let alloc_calls_ratio = ratio(
        use_vec_allocs.alloc_calls as f64,
        immutable_allocs.alloc_calls as f64,
    );
    let alloc_bytes_ratio = ratio(
        use_vec_allocs.gross_allocated_bytes() as f64,
        immutable_allocs.gross_allocated_bytes() as f64,
    );
    println!();
    println!("{:<38}: {:.2}x", "use_vec time vs immutable", time_ratio);
    println!(
        "{:<38}: {:.2}x",
        "use_vec allocation calls vs immutable", alloc_calls_ratio
    );
    println!(
        "{:<38}: {:.2}x",
        "use_vec allocated bytes vs immutable", alloc_bytes_ratio
    );

    println!();

    println!("immutable best distance: {immutable_best:.6}");
    println!("use_vec best distance:   {use_vec_best:.6}");

    if (immutable_best - use_vec_best).abs() > 1e-9 {
        println!(
            "warning: best distances differ by {:.6}",
            (immutable_best - use_vec_best).abs()
        );
    }
}
