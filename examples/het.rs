use clap::Parser;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{hint::black_box, time::Instant};

const NUM_THREADS: usize = 16;

#[derive(Parser, Debug)]
struct Args {
    /// Input length exponent n where len = 2^n.
    #[arg(long, default_value_t = 14)]
    len_exp: usize,

    /// Probability of selecting a heavy task in [0.0, 1.0].
    #[arg(long, default_value_t = 0.101)]
    heterogeneity_level: f64,
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

fn heterogeneous_map(heterogeneity_level: f64, i: u64) -> u64 {
    let mut rng = ChaCha8Rng::seed_from_u64(i);
    for _ in 0..10 * i {
        let _: u32 = rng.random();
    }

    let n = if rng.random_bool(heterogeneity_level) {
        200_000_000
    } else {
        1
    };

    fibonacci(n)
}

fn run_seq(input: &[u64], heterogeneity_level: f64) -> Option<u64> {
    input
        .iter()
        .map(|x| heterogeneous_map(heterogeneity_level, *x))
        .max()
}

fn run_rayon(
    pool: &rayon_core::ThreadPool,
    input: &[u64],
    heterogeneity_level: f64,
) -> Option<u64> {
    pool.install(|| {
        input
            .par_iter()
            .map(|x| heterogeneous_map(heterogeneity_level, *x))
            .max()
    })
}

fn run_orx_fixed(
    pool: &rayon_core::ThreadPool,
    input: &[u64],
    heterogeneity_level: f64,
) -> Option<u64> {
    input
        .into_par()
        .runner(Runner::fixed_chunk(pool))
        .num_threads(NUM_THREADS)
        .map(|x| heterogeneous_map(heterogeneity_level, *x))
        .max()
}

fn run_timed(name: &str, f: impl FnOnce() -> Option<u64>) -> (Option<u64>, f64) {
    let start = Instant::now();
    let out = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
    println!("{name:<10} | {elapsed_ms:>9.3} ms | output={out:?}");
    (out, elapsed_ms)
}

fn main() {
    let args = Args::parse();

    assert!(args.len_exp < usize::BITS as usize, "len_exp is too large");
    assert!(
        (0.0..=1.0).contains(&args.heterogeneity_level),
        "heterogeneity_level must be in [0.0, 1.0]"
    );

    let len = 1usize << args.len_exp;
    let input: Vec<u64> = (0..len).map(|i| i as u64).collect();

    println!(
        "het example: len_exp={} len={} heterogeneity_level={} threads={}",
        args.len_exp, len, args.heterogeneity_level, NUM_THREADS
    );

    let pool = Pool::rayon(NUM_THREADS).expect("failed to build rayon thread pool");

    let (seq, _) = run_timed("seq", || run_seq(&input, args.heterogeneity_level));
    let (rayon, _) = run_timed("rayon", || {
        run_rayon(&pool, &input, args.heterogeneity_level)
    });
    let (orx_fixed, _) = run_timed("orx-fixed", || {
        run_orx_fixed(&pool, &input, args.heterogeneity_level)
    });

    assert_eq!(rayon, seq, "rayon output mismatch");
    assert_eq!(orx_fixed, seq, "orx-fixed output mismatch");

    println!("all methods produced identical outputs");
}
