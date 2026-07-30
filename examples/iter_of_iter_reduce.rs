use clap::Parser;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Method {
    Seq,
    Orx,
}

#[derive(Parser)]
struct Args {
    /// Parallelization method to use
    #[arg(long, default_value = "orx")]
    method: Method,
    /// Number of elements (as power of 2, e.g. 20 means 2^20 = ~1M)
    #[arg(long, default_value_t = 20)]
    n: u32,
    /// Number of threads (0 = auto)
    #[arg(long, default_value_t = 4)]
    num_threads: usize,
    /// Number of warmup runs
    #[arg(long, default_value_t = 3)]
    warmup: usize,
    /// Number of timed runs to average
    #[arg(long, default_value_t = 10)]
    runs: usize,
    /// Print orx diagnostics after first timed run
    #[arg(long)]
    diagnostics: bool,
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 0xBEEF_CAFE_1234_5678;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len).map(|_| rng.random_range(0..150_u64)).collect()
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 0..n {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    a
}

fn map_item(x: &u64) -> u64 {
    7 * x + 1000
}

fn reduce_pair(a: u64, b: u64) -> u64 {
    let f = fibonacci(a % 5);
    let g = a.wrapping_add(f);
    g.wrapping_add(b).wrapping_sub(f)
}

fn run_seq(input: &[u64]) -> Option<u64> {
    black_box(input)
        .iter()
        .map(map_item)
        .filter(|&x| x % 2 == 0)
        .reduce(reduce_pair)
}

fn run_orx(input: &[u64], num_threads: usize, diag: bool) -> Option<u64> {
    let par = black_box(input)
        .iter()
        .iter_into_par()
        .map(map_item)
        .filter(|&x| x % 2 == 0)
        .num_threads(num_threads);

    match diag {
        false => par.reduce(reduce_pair),
        true => par.runner_with_diagnostics().reduce(reduce_pair),
    }
}

fn run_once(input: &[u64], method: Method, num_threads: usize, diag: bool) -> Option<u64> {
    match method {
        Method::Seq => run_seq(input),
        Method::Orx => run_orx(input, num_threads, diag),
    }
}

fn main() {
    let args = Args::parse();
    let len = 1usize << args.n;

    println!(
        "method={:?}  n=2^{}={}  num_threads={}  warmup={}  runs={}  diagnostics={}",
        args.method, args.n, len, args.num_threads, args.warmup, args.runs, args.diagnostics
    );

    println!("\nGenerating {} inputs...", len);
    let input = inputs(len);

    println!("Warming up ({} runs)...", args.warmup);
    for _ in 0..args.warmup {
        let _ = black_box(run_once(&input, args.method, args.num_threads, false));
    }

    println!("Running {} timed iterations...", args.runs);
    let expected = run_once(&input, Method::Seq, args.num_threads, false);
    let mut durations: Vec<Duration> = Vec::with_capacity(args.runs);
    let mut result: Option<u64> = None;

    for i in 0..args.runs {
        let diag = args.diagnostics && i == 0;
        let t0 = Instant::now();
        result = black_box(run_once(&input, args.method, args.num_threads, diag));
        durations.push(t0.elapsed());
    }
    assert_eq!(result, expected);

    let total_ns: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
    let avg_ns = total_ns / args.runs as u64;
    let min_ns = durations.iter().map(|d| d.as_nanos() as u64).min().unwrap();
    let max_ns = durations.iter().map(|d| d.as_nanos() as u64).max().unwrap();

    println!("\n--- Results ---");
    println!("avg: {:.3} ms", avg_ns as f64 / 1_000_000.0);
    println!("min: {:.3} ms", min_ns as f64 / 1_000_000.0);
    println!("max: {:.3} ms", max_ns as f64 / 1_000_000.0);
    println!("output: {:?}", result);
}
