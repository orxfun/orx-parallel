use clap::{ArgAction, Parser, ValueEnum};
use orx_parallel::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::hint::black_box;
use std::time::Instant;

const NORMAL_ITERS_LIGHT: u32 = 200;
const NORMAL_ITERS_HEAVY: u32 = 50;
const OUTLIER_MULTIPLIER_MIN: u32 = 10000;
const OUTLIER_MULTIPLIER_MAX: u32 = 100000;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum Method {
    Seq,
    Rayon,
    FixedAuto,
    Fixed1,
    B,
}

#[derive(Parser, Debug)]
struct Args {
    /// Number of input elements.
    #[arg(long, default_value_t = 1 << 10)]
    n: usize,

    /// Number of threads for rayon and orx methods.
    #[arg(long, default_value_t = 8)]
    num_threads: usize,

    /// Percentage of outlier elements in [0, 100].
    #[arg(long, default_value_t = 1.0)]
    heterogeneity_percent: f64,

    /// Whether normal elements are moderate-cost (true) or very fast (false).
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    heavy: bool,

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
}

fn seed_for_input(n: usize, heavy: bool, heterogeneity_percent: f64) -> u64 {
    let heavy_flag = if heavy { 0xC3u64 } else { 0xD4u64 };
    let het_scaled = (heterogeneity_percent * 10_000.0).round() as u64;
    0xC0FF_EE12_3456_7890u64 ^ ((n as u64) << 16) ^ (heavy_flag << 8) ^ het_scaled
}

fn build_workload(n: usize, heavy: bool, heterogeneity_percent: f64) -> Vec<WorkItem> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed_for_input(n, heavy, heterogeneity_percent));

    let normal_iters = if heavy {
        NORMAL_ITERS_HEAVY
    } else {
        NORMAL_ITERS_LIGHT
    };

    let mut items = vec![
        WorkItem {
            seed: 0,
            iterations: normal_iters,
        };
        n
    ];

    for (idx, item) in items.iter_mut().enumerate() {
        item.seed = idx as u64 ^ rng.random::<u64>();
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

fn run_seq(input: &[WorkItem]) -> Option<u64> {
    input.iter().map(do_work).max()
}

fn run_rayon(input: &[WorkItem], num_threads: usize) -> Option<u64> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("failed to build rayon thread pool");

    pool.install(|| input.par_iter().map(do_work).max())
}

fn run_orx_fixed_auto(input: &[WorkItem], num_threads: usize, diagnostics: bool) -> Option<u64> {
    let par = input
        .par()
        .num_threads(num_threads)
        .chunk_size(0)
        .runner(Runner::fixed_chunk(Pool::once(num_threads)))
        .map(do_work);

    if diagnostics {
        par.runner_with_diagnostics().max()
    } else {
        par.max()
    }
}

fn run_orx_fixed_1(input: &[WorkItem], num_threads: usize, diagnostics: bool) -> Option<u64> {
    let par = input
        .par()
        .num_threads(num_threads)
        .chunk_size(1)
        .runner(Runner::fixed_chunk(Pool::once(num_threads)))
        .map(do_work);

    if diagnostics {
        par.runner_with_diagnostics().max()
    } else {
        par.max()
    }
}

fn run_orx_b(input: &[WorkItem], num_threads: usize, diagnostics: bool) -> Option<u64> {
    let par = input
        .par()
        .num_threads(num_threads)
        .chunk_size(0)
        .runner(Runner::b(Pool::once(num_threads)))
        .map(do_work);

    if diagnostics {
        par.runner_with_diagnostics().max()
    } else {
        par.max()
    }
}

fn run_selected_method(args: &Args, input: &[WorkItem]) -> Option<u64> {
    match args.method {
        Method::Seq => run_seq(input),
        Method::Rayon => run_rayon(input, args.num_threads),
        Method::FixedAuto => run_orx_fixed_auto(input, args.num_threads, args.diagnostics),
        Method::Fixed1 => run_orx_fixed_1(input, args.num_threads, args.diagnostics),
        Method::B => run_orx_b(input, args.num_threads, args.diagnostics),
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

    let input = build_workload(args.n, args.heavy, args.heterogeneity_percent);
    let expected = run_seq(&input);

    let start = Instant::now();
    let output = run_selected_method(&args, &input);
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    assert_eq!(output, expected, "output mismatch");

    println!(
        "method={:?} n={} threads={} heavy={} heterogeneity_percent={} diagnostics={} elapsed_ms={:.3}",
        args.method,
        args.n,
        args.num_threads,
        args.heavy,
        args.heterogeneity_percent,
        args.diagnostics,
        elapsed_ms,
    );
}
