mod alloc_tracking;
mod locations;
mod par_with_use;
mod par_without_use;
mod rand_utils;

use crate::par_with_use::run_search_parallel_use_mut;
use crate::par_without_use::run_search_parallel_immutable;
use crate::{alloc_tracking::AllocationStats, locations::locations};
use std::hint::black_box;
use std::time::{Duration, Instant};

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

fn main() {
    let iterations = 10000;
    let threads = 4;
    let num_cities = 200;
    let seed = 42;
    let rounds = 1;

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
    println!("immutable avg: {:.3} ms", immutable_avg.as_secs_f64() * 1e3);
    println!("use_vec avg:   {:.3} ms", use_vec_avg.as_secs_f64() * 1e3);
    println!(
        "immutable avg allocations: {} calls, {} bytes (gross)",
        immutable_allocs.alloc_calls,
        immutable_allocs.gross_allocated_bytes()
    );
    println!(
        "use_vec avg allocations:   {} calls, {} bytes (gross)",
        use_vec_allocs.alloc_calls,
        use_vec_allocs.gross_allocated_bytes()
    );
    println!(
        "immutable avg releases: {} calls, {} bytes (gross)",
        immutable_allocs.dealloc_calls + immutable_allocs.realloc_calls,
        immutable_allocs.gross_released_bytes()
    );
    println!(
        "use_vec avg releases:   {} calls, {} bytes (gross)",
        use_vec_allocs.dealloc_calls + use_vec_allocs.realloc_calls,
        use_vec_allocs.gross_released_bytes()
    );

    let speedup = immutable_avg.as_secs_f64() / use_vec_avg.as_secs_f64();
    println!("use_vec speedup vs immutable: {speedup:.2}x");

    println!("immutable best distance: {immutable_best:.6}");
    println!("use_vec best distance:   {use_vec_best:.6}");

    if (immutable_best - use_vec_best).abs() > 1e-9 {
        println!(
            "warning: best distances differ by {:.6}",
            (immutable_best - use_vec_best).abs()
        );
    }
}
