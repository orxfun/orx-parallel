use clap::Parser;
use orx_parallel::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{hint::black_box, time::Instant};

const NUM_THREADS: usize = 16;

#[derive(Parser, Debug)]
struct Args {
    /// Input length exponent n where len = 2^n.
    #[arg(long, default_value_t = 9)]
    len_exp: usize,

    /// Probability of selecting a heavy task in [0.0, 1.0].
    #[arg(long, default_value_t = 0.1)]
    heterogeneity_level: f64,

    /// Chunk size parameter to be used by orx variants (0: Auto).
    #[arg(long, default_value_t = 0)]
    chunk_size: usize,

    /// Number of timed repetitions for parallel variants.
    #[arg(long, default_value_t = 7)]
    num_repetitions: usize,

    /// Number of warmup repetitions for parallel variants; excluded from stats.
    #[arg(long, default_value_t = 1)]
    num_warmup: usize,

    /// Base seed used to derive per-repetition workload seeds and method order.
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Method {
    Rayon,
    OrxAdaptive,
    OrxFixed,
}

impl Method {
    fn name(self) -> &'static str {
        match self {
            Method::Rayon => "rayon",
            Method::OrxAdaptive => "orx-adapt",
            Method::OrxFixed => "orx-fixed",
        }
    }
}

#[derive(Debug, Default)]
struct Stats {
    times_ms: Vec<f64>,
}

impl Stats {
    fn push(&mut self, elapsed_ms: f64) {
        self.times_ms.push(elapsed_ms);
    }

    fn mean_ms(&self) -> f64 {
        self.times_ms.iter().sum::<f64>() / self.times_ms.len() as f64
    }

    fn min_ms(&self) -> f64 {
        self.times_ms.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max_ms(&self) -> f64 {
        self.times_ms
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    fn median_ms(&self) -> f64 {
        let mut sorted = self.times_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("times must be finite"));
        let n = sorted.len();
        if n % 2 == 1 {
            sorted[n / 2]
        } else {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        }
    }
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = black_box(a + b);
        a = b;
        b = c;
    }
    a
}

fn heterogeneous_map(heterogeneity_level: f64, workload_seed: u64, i: u64) -> u64 {
    let mut rng = ChaCha8Rng::seed_from_u64(workload_seed ^ i);
    for _ in 0..10 * i {
        let _: u32 = rng.random();
    }

    let n = match rng.random_bool(heterogeneity_level) {
        true => 200_000_000,
        false => 1,
    };

    fibonacci(n)
}

fn run_seq(input: &[u64], heterogeneity_level: f64, workload_seed: u64) -> Option<u64> {
    input
        .iter()
        .map(|x| heterogeneous_map(heterogeneity_level, workload_seed, *x))
        .max()
}

fn run_rayon(
    pool: &rayon_core::ThreadPool,
    input: &[u64],
    heterogeneity_level: f64,
    workload_seed: u64,
) -> Option<u64> {
    pool.install(|| {
        input
            .par_iter()
            .map(|x| heterogeneous_map(heterogeneity_level, workload_seed, *x))
            .max()
    })
}

fn run_orx(
    pool: &rayon_core::ThreadPool,
    input: &[u64],
    heterogeneity_level: f64,
    chunk_size: usize,
    adaptive: bool,
    workload_seed: u64,
) -> Option<u64> {
    let par = input
        .into_par()
        .num_threads(NUM_THREADS)
        .chunk_size(chunk_size)
        .map(|x| heterogeneous_map(heterogeneity_level, workload_seed, *x));
    match adaptive {
        true => par.runner(Runner::adaptive(pool)).max(),
        false => par.runner(Runner::fixed(pool)).max(),
    }
}

fn run_timed(name: &str, f: impl FnOnce() -> Option<u64>) -> (Option<u64>, f64) {
    let start = Instant::now();
    let out = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
    println!("{name:<10} | {elapsed_ms:>9.3} ms | output={out:?}");
    (out, elapsed_ms)
}

fn run_once(
    method: Method,
    pool: &rayon_core::ThreadPool,
    input: &[u64],
    heterogeneity_level: f64,
    chunk_size: usize,
    workload_seed: u64,
) -> (Option<u64>, f64) {
    let start = Instant::now();
    let out = match method {
        Method::Rayon => run_rayon(pool, input, heterogeneity_level, workload_seed),
        Method::OrxAdaptive => run_orx(
            pool,
            input,
            heterogeneity_level,
            chunk_size,
            true,
            workload_seed,
        ),
        Method::OrxFixed => run_orx(
            pool,
            input,
            heterogeneity_level,
            chunk_size,
            false,
            workload_seed,
        ),
    };
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
    (out, elapsed_ms)
}

fn main() {
    let args = Args::parse();

    assert!(args.len_exp < usize::BITS as usize, "len_exp is too large");
    assert!(
        (0.0..=1.0).contains(&args.heterogeneity_level),
        "heterogeneity_level must be in [0.0, 1.0]"
    );
    assert!(args.num_repetitions > 0, "num_repetitions must be positive");

    let len = 1usize << args.len_exp;
    let input: Vec<u64> = (0..len).map(|i| i as u64).collect();

    println!(
        "heterogeneous computation benchmark: len_exp={} len={} heterogeneity_level={} threads={} chunk_size={} reps={} warmup={}",
        args.len_exp,
        len,
        args.heterogeneity_level,
        NUM_THREADS,
        args.chunk_size,
        args.num_repetitions,
        args.num_warmup
    );

    let pool = Pool::rayon(NUM_THREADS).expect("failed to build rayon thread pool");

    // Sequential baseline is intentionally run once because it is much slower.
    let seq_seed = args.seed;
    let (seq, _) = run_timed("seq", || {
        run_seq(&input, args.heterogeneity_level, seq_seed)
    });

    let methods = vec![Method::Rayon, Method::OrxAdaptive, Method::OrxFixed];

    for warmup in 0..args.num_warmup {
        let workload_seed = args.seed.wrapping_add(10_000 + warmup as u64);
        for method in &methods {
            let _ = run_once(
                *method,
                &pool,
                &input,
                args.heterogeneity_level,
                args.chunk_size,
                workload_seed,
            );
        }
    }

    let mut stats: Vec<(Method, Stats)> = methods
        .iter()
        .copied()
        .map(|m| (m, Stats::default()))
        .collect();

    let mut order_rng = ChaCha8Rng::seed_from_u64(args.seed.wrapping_add(999_999));

    for rep in 0..args.num_repetitions {
        let workload_seed = args.seed.wrapping_add(rep as u64);
        let mut run_order = methods.clone();
        run_order.shuffle(&mut order_rng);

        println!(
            "\nrep {:>2} | workload_seed={} | order={:?}",
            rep + 1,
            workload_seed,
            run_order
        );

        let mut rep_results: Vec<(Method, Option<u64>)> = Vec::with_capacity(run_order.len());

        for method in run_order {
            let (out, elapsed_ms) = run_once(
                method,
                &pool,
                &input,
                args.heterogeneity_level,
                args.chunk_size,
                workload_seed,
            );
            println!(
                "  {:<10} | {:>9.3} ms | output={out:?}",
                method.name(),
                elapsed_ms
            );

            if let Some((_, s)) = stats.iter_mut().find(|(m, _)| *m == method) {
                s.push(elapsed_ms);
            }
            rep_results.push((method, out));
        }

        let expected = rep_results[0].1;
        for (method, out) in rep_results.iter().skip(1) {
            assert_eq!(
                *out,
                expected,
                "output mismatch in rep {} for method {}",
                rep + 1,
                method.name()
            );
        }

        if rep == 0 {
            assert_eq!(
                expected, seq,
                "parallel output mismatch against sequential baseline in rep 1"
            );
        }
    }

    println!("\nsummary (ms):");
    println!(
        "{: <10} | {:>9} | {:>9} | {:>9} | {:>9}",
        "method", "mean", "median", "min", "max"
    );
    println!("{}", "-".repeat(60));
    for (method, s) in &stats {
        println!(
            "{: <10} | {:>9.3} | {:>9.3} | {:>9.3} | {:>9.3}",
            method.name(),
            s.mean_ms(),
            s.median_ms(),
            s.min_ms(),
            s.max_ms(),
        );
    }

    println!("\nall methods produced identical outputs");
}
