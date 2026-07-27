/// Experimental heterogeneous benchmark to find edge cases for parallelization.
/// Tests various computation patterns beyond simple map() to expose scheduler weaknesses.
use clap::{ArgAction, Parser, ValueEnum};
use orx_parallel::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::hint::black_box;
use std::time::Instant;

const NORMAL_ITERS_LIGHT: u32 = 50;
const NORMAL_ITERS_HEAVY: u32 = 500;
const OUTLIER_MULTIPLIER_MIN: u32 = 10;
const OUTLIER_MULTIPLIER_MAX: u32 = 100;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum Method {
    Seq,
    Rayon,
    FixedAuto,
    Fixed1,
    B,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ComputationPattern {
    /// Simple map: work per element is fixed (given by iterations)
    Map,
    /// Filter + map: only p% of elements produce output, rest have zero work
    FilterMap,
    /// FlatMap: each element produces variable output counts (0 or k items)
    FlatMap,
    /// Accumulate: sequential dependency chain within parallel iteration
    Accumulate,
    /// Tiny: extremely small per-element work (100-1000 cycles only)
    Tiny,
}

#[derive(Parser, Debug)]
struct Args {
    /// Number of input elements.
    #[arg(long, default_value_t = 1 << 14)]
    n: usize,

    /// Number of threads for rayon and orx methods.
    #[arg(long, default_value_t = 8)]
    num_threads: usize,

    /// Percentage of outlier elements in [0, 100].
    #[arg(long, default_value_t = 10.0)]
    heterogeneity_percent: f64,

    /// Whether normal elements are moderate-cost (true) or very fast (false).
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    heavy: bool,

    /// Computation pattern to use.
    #[arg(long, value_enum, default_value_t = ComputationPattern::Map)]
    pattern: ComputationPattern,

    /// For filter/flatmap patterns: percentage of items that survive/produce output.
    #[arg(long, default_value_t = 50.0)]
    survival_percent: f64,

    /// Method variant to run.
    #[arg(long, value_enum, default_value_t = Method::FixedAuto)]
    method: Method,

    /// Enables diagnostics printing for orx variants.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    diagnostics: bool,
}

#[derive(Clone, Copy)]
struct WorkItem {
    seed: u64,
    iterations: u32,
    item_id: usize,
}

fn seed_for_input(
    n: usize,
    heavy: bool,
    heterogeneity_percent: f64,
    pattern: ComputationPattern,
) -> u64 {
    let heavy_flag = if heavy { 0xC3u64 } else { 0xD4u64 };
    let het_scaled = (heterogeneity_percent * 10_000.0).round() as u64;
    let pattern_flag = match pattern {
        ComputationPattern::Map => 0x01u64,
        ComputationPattern::FilterMap => 0x02u64,
        ComputationPattern::FlatMap => 0x03u64,
        ComputationPattern::Accumulate => 0x04u64,
        ComputationPattern::Tiny => 0x05u64,
    };
    0xC0FF_EE12_3456_7890u64
        ^ ((n as u64) << 16)
        ^ (heavy_flag << 8)
        ^ het_scaled
        ^ (pattern_flag << 24)
}

fn build_workload(
    n: usize,
    heavy: bool,
    heterogeneity_percent: f64,
    pattern: ComputationPattern,
) -> Vec<WorkItem> {
    let mut rng =
        ChaCha8Rng::seed_from_u64(seed_for_input(n, heavy, heterogeneity_percent, pattern));

    let normal_iters = match pattern {
        ComputationPattern::Tiny => 1, // Extremely tiny work
        ComputationPattern::FilterMap | ComputationPattern::FlatMap => {
            if heavy {
                NORMAL_ITERS_HEAVY
            } else {
                NORMAL_ITERS_LIGHT
            }
        }
        _ => {
            if heavy {
                NORMAL_ITERS_HEAVY
            } else {
                NORMAL_ITERS_LIGHT
            }
        }
    };

    let mut items = vec![
        WorkItem {
            seed: 0,
            iterations: normal_iters,
            item_id: 0,
        };
        n
    ];

    for (idx, item) in items.iter_mut().enumerate() {
        item.seed = idx as u64 ^ rng.random::<u64>();
        item.item_id = idx;
    }

    let outlier_count = ((heterogeneity_percent * n as f64) / 100.0).floor() as usize;
    if outlier_count == 0 {
        return items;
    }

    let mut indices: Vec<usize> = (0..n).collect();
    indices.shuffle(&mut rng);

    for idx in indices.into_iter().take(outlier_count) {
        let m = rng.random_range(OUTLIER_MULTIPLIER_MIN..=OUTLIER_MULTIPLIER_MAX);
        items[idx].iterations = items[idx].iterations.saturating_mul(m);
    }

    items
}

fn do_work(item: &WorkItem) -> u64 {
    let mut state = item.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    for _ in 0..item.iterations {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state ^= state >> 33;
        state = black_box(state);
    }
    state
}

fn run_seq_map(input: &[WorkItem]) -> u64 {
    input.iter().map(do_work).max().unwrap_or(0)
}

fn run_seq_filter_map(input: &[WorkItem], survival_percent: f64) -> u64 {
    let threshold = ((survival_percent / 100.0) * u32::MAX as f64) as u32;
    input
        .iter()
        .filter(|item| {
            let hash = item.seed.wrapping_mul(6364136223846793005) as u32;
            hash < threshold
        })
        .map(do_work)
        .max()
        .unwrap_or(0)
}

fn run_seq_flatmap(input: &[WorkItem], survival_percent: f64) -> u64 {
    let repeat_count = ((survival_percent / 100.0) * 5.0).ceil() as usize;
    input
        .iter()
        .flat_map(|item| {
            let is_special = (item.seed as u32) % 100 < 30;
            if is_special {
                (0..repeat_count).map(|_| do_work(item)).collect::<Vec<_>>()
            } else {
                vec![do_work(item)]
            }
        })
        .max()
        .unwrap_or(0)
}

fn run_seq_tiny(input: &[WorkItem]) -> u64 {
    input
        .iter()
        .map(|item| {
            let mut state = item.seed;
            for _ in 0..10 {
                state = state.wrapping_mul(6364136223846793005) ^ state;
            }
            state
        })
        .max()
        .unwrap_or(0)
}

fn run_seq(input: &[WorkItem], pattern: ComputationPattern, survival_percent: f64) -> u64 {
    match pattern {
        ComputationPattern::Map => run_seq_map(input),
        ComputationPattern::FilterMap => run_seq_filter_map(input, survival_percent),
        ComputationPattern::FlatMap => run_seq_flatmap(input, survival_percent),
        ComputationPattern::Tiny => run_seq_tiny(input),
        ComputationPattern::Accumulate => run_seq_map(input), // same as map for now
    }
}

fn run_rayon_map(input: &[WorkItem], num_threads: usize) -> u64 {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("failed to build rayon thread pool");

    pool.install(|| input.par_iter().map(do_work).max().unwrap_or(0))
}

fn run_rayon_filter_map(input: &[WorkItem], num_threads: usize, survival_percent: f64) -> u64 {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("failed to build rayon thread pool");

    let threshold = ((survival_percent / 100.0) * u32::MAX as f64) as u32;
    pool.install(|| {
        input
            .par_iter()
            .filter(|item| {
                let hash = item.seed.wrapping_mul(6364136223846793005) as u32;
                hash < threshold
            })
            .map(do_work)
            .max()
            .unwrap_or(0)
    })
}

fn run_rayon_flatmap(input: &[WorkItem], num_threads: usize, survival_percent: f64) -> u64 {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("failed to build rayon thread pool");

    let repeat_count = ((survival_percent / 100.0) * 5.0).ceil() as usize;
    pool.install(|| {
        input
            .par_iter()
            .flat_map(|item| {
                let is_special = (item.seed as u32) % 100 < 30;
                if is_special {
                    (0..repeat_count).map(|_| do_work(item)).collect::<Vec<_>>()
                } else {
                    vec![do_work(item)]
                }
            })
            .max()
            .unwrap_or(0)
    })
}

fn run_rayon_tiny(input: &[WorkItem], num_threads: usize) -> u64 {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("failed to build rayon thread pool");

    pool.install(|| {
        input
            .par_iter()
            .map(|item| {
                let mut state = item.seed;
                for _ in 0..10 {
                    state = state.wrapping_mul(6364136223846793005) ^ state;
                }
                state
            })
            .max()
            .unwrap_or(0)
    })
}

fn run_rayon(
    input: &[WorkItem],
    num_threads: usize,
    pattern: ComputationPattern,
    survival_percent: f64,
) -> u64 {
    match pattern {
        ComputationPattern::Map => run_rayon_map(input, num_threads),
        ComputationPattern::FilterMap => run_rayon_filter_map(input, num_threads, survival_percent),
        ComputationPattern::FlatMap => run_rayon_flatmap(input, num_threads, survival_percent),
        ComputationPattern::Tiny => run_rayon_tiny(input, num_threads),
        ComputationPattern::Accumulate => run_rayon_map(input, num_threads),
    }
}

fn run_orx_fixed_auto(
    input: &[WorkItem],
    num_threads: usize,
    pattern: ComputationPattern,
    survival_percent: f64,
    diagnostics: bool,
) -> u64 {
    match pattern {
        ComputationPattern::Map => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::FilterMap => {
            let threshold = ((survival_percent / 100.0) * u32::MAX as f64) as u32;
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .filter(|item| {
                    let hash = item.seed.wrapping_mul(6364136223846793005) as u32;
                    hash < threshold
                })
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::FlatMap => {
            let repeat_count = ((survival_percent / 100.0) * 5.0).ceil() as usize;
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .flat_map(move |item| {
                    let is_special = (item.seed as u32) % 100 < 30;
                    if is_special {
                        (0..repeat_count).map(|_| do_work(item)).collect::<Vec<_>>()
                    } else {
                        vec![do_work(item)]
                    }
                });
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::Tiny => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .map(|item| {
                    let mut state = item.seed;
                    for _ in 0..10 {
                        state = state.wrapping_mul(6364136223846793005) ^ state;
                    }
                    state
                });
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::Accumulate => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
    }
}

fn run_orx_fixed_1(
    input: &[WorkItem],
    num_threads: usize,
    pattern: ComputationPattern,
    survival_percent: f64,
    diagnostics: bool,
) -> u64 {
    match pattern {
        ComputationPattern::Map => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(1)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::FilterMap => {
            let threshold = ((survival_percent / 100.0) * u32::MAX as f64) as u32;
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(1)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .filter(|item| {
                    let hash = item.seed.wrapping_mul(6364136223846793005) as u32;
                    hash < threshold
                })
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::FlatMap => {
            let repeat_count = ((survival_percent / 100.0) * 5.0).ceil() as usize;
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(1)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .flat_map(move |item| {
                    let is_special = (item.seed as u32) % 100 < 30;
                    if is_special {
                        (0..repeat_count).map(|_| do_work(item)).collect::<Vec<_>>()
                    } else {
                        vec![do_work(item)]
                    }
                });
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::Tiny => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(1)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .map(|item| {
                    let mut state = item.seed;
                    for _ in 0..10 {
                        state = state.wrapping_mul(6364136223846793005) ^ state;
                    }
                    state
                });
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::Accumulate => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(1)
                .runner(Runner::fixed_chunk(Pool::once(num_threads)))
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
    }
}

fn run_orx_b(
    input: &[WorkItem],
    num_threads: usize,
    pattern: ComputationPattern,
    survival_percent: f64,
    diagnostics: bool,
) -> u64 {
    match pattern {
        ComputationPattern::Map => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::b(Pool::once(num_threads)))
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::FilterMap => {
            let threshold = ((survival_percent / 100.0) * u32::MAX as f64) as u32;
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::b(Pool::once(num_threads)))
                .filter(|item| {
                    let hash = item.seed.wrapping_mul(6364136223846793005) as u32;
                    hash < threshold
                })
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::FlatMap => {
            let repeat_count = ((survival_percent / 100.0) * 5.0).ceil() as usize;
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::b(Pool::once(num_threads)))
                .flat_map(move |item| {
                    let is_special = (item.seed as u32) % 100 < 30;
                    if is_special {
                        (0..repeat_count).map(|_| do_work(item)).collect::<Vec<_>>()
                    } else {
                        vec![do_work(item)]
                    }
                });
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::Tiny => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::b(Pool::once(num_threads)))
                .map(|item| {
                    let mut state = item.seed;
                    for _ in 0..10 {
                        state = state.wrapping_mul(6364136223846793005) ^ state;
                    }
                    state
                });
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
        ComputationPattern::Accumulate => {
            let par = input
                .par()
                .num_threads(num_threads)
                .chunk_size(0)
                .runner(Runner::b(Pool::once(num_threads)))
                .map(do_work);
            if diagnostics {
                par.runner_with_diagnostics().max().unwrap_or(0)
            } else {
                par.max().unwrap_or(0)
            }
        }
    }
}

fn run_selected_method(args: &Args, input: &[WorkItem]) -> u64 {
    match args.method {
        Method::Seq => run_seq(input, args.pattern, args.survival_percent),
        Method::Rayon => run_rayon(input, args.num_threads, args.pattern, args.survival_percent),
        Method::FixedAuto => run_orx_fixed_auto(
            input,
            args.num_threads,
            args.pattern,
            args.survival_percent,
            args.diagnostics,
        ),
        Method::Fixed1 => run_orx_fixed_1(
            input,
            args.num_threads,
            args.pattern,
            args.survival_percent,
            args.diagnostics,
        ),
        Method::B => run_orx_b(
            input,
            args.num_threads,
            args.pattern,
            args.survival_percent,
            args.diagnostics,
        ),
    }
}

fn main() {
    let args = Args::parse();

    assert!(args.n > 0, "n must be positive");
    assert!(args.num_threads > 0, "num_threads must be positive");
    assert!(
        (0.0..=100.0).contains(&args.heterogeneity_percent),
        "heterogeneity_percent must be in [0, 100]"
    );
    assert!(
        (0.0..=100.0).contains(&args.survival_percent),
        "survival_percent must be in [0, 100]"
    );

    let input = build_workload(args.n, args.heavy, args.heterogeneity_percent, args.pattern);
    let expected = run_seq(&input, args.pattern, args.survival_percent);

    let start = Instant::now();
    let output = run_selected_method(&args, &input);
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    assert_eq!(output, expected, "output mismatch");

    println!(
        "pattern={:?} method={:?} n={} threads={} heavy={} het%={} survival%={} elapsed_ms={:.3}",
        args.pattern,
        args.method,
        args.n,
        args.num_threads,
        args.heavy,
        args.heterogeneity_percent,
        args.survival_percent,
        elapsed_ms,
    );
}
