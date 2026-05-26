use clap::Parser;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::hint::black_box;
use std::time::Instant;

const NUM_THREADS: usize = 16;
const HOMOGENEOUS_WORK: usize = 96;
const HETEROGENEOUS_LIGHT_WORK: usize = 24;
const HETEROGENEOUS_MEDIUM_WORK: usize = 192;
const HETEROGENEOUS_HEAVY_WORK: usize = 1536;

#[derive(Clone, Copy, Debug)]
enum TaskKind {
    Homogeneous,
    Heterogeneous,
}

impl TaskKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Homogeneous => "homogeneous",
            Self::Heterogeneous => "heterogeneous",
        }
    }
}

impl core::str::FromStr for TaskKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "homogeneous" => Ok(Self::Homogeneous),
            "heterogeneous" => Ok(Self::Heterogeneous),
            _ => Err(format!(
                "unknown task kind '{s}', expected homogeneous or heterogeneous"
            )),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Task profile: homogeneous or heterogeneous.
    #[arg(long, default_value = "heterogeneous")]
    task_kind: TaskKind,

    /// Input length exponent n where len = 2^n.
    #[arg(long, default_value_t = 20)]
    len_exp: usize,
}

fn values(len: usize) -> Vec<u64> {
    const SEED: u64 = 0xD1CE_BA5E;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len)
        .map(|idx| rng.random::<u64>() ^ ((idx as u64 + 1) * 0x9E37_79B9_7F4A_7C15))
        .collect()
}

fn work_units(value: u64, task_kind: TaskKind) -> usize {
    match task_kind {
        TaskKind::Homogeneous => HOMOGENEOUS_WORK,
        TaskKind::Heterogeneous => match value & 0x0f {
            0 => HETEROGENEOUS_HEAVY_WORK,
            1 | 2 | 3 => HETEROGENEOUS_MEDIUM_WORK,
            _ => HETEROGENEOUS_LIGHT_WORK,
        },
    }
}

fn expensive_map(value: &u64, task_kind: TaskKind) -> u64 {
    let mut acc = black_box(*value ^ 0xA076_1D64_78BD_642F);
    let rounds = work_units(*value, task_kind);

    for round in 0..rounds {
        let salt = black_box((round as u64 + 1) * 0xE703_7ED1_A0B4_28DB);
        acc = acc.rotate_left(11) ^ salt;
        acc = acc.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        acc ^= acc >> 29;
    }

    acc ^ (*value).rotate_left(7)
}

fn selective_filter(value: &u64) -> bool {
    let folded = value ^ value.rotate_right(17);
    folded.count_ones() % 3 != 0
}

fn reduce_sum(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

fn run_seq(data: &[u64], task_kind: TaskKind) -> Option<u64> {
    data.iter()
        .map(|value| expensive_map(value, task_kind))
        .filter(selective_filter)
        .reduce(reduce_sum)
}

fn run_rayon(pool: &rayon_core::ThreadPool, data: &[u64], task_kind: TaskKind) -> Option<u64> {
    pool.install(|| {
        data.par_iter()
            .map(|value| expensive_map(value, task_kind))
            .filter(selective_filter)
            .reduce_with(reduce_sum)
    })
}

fn run_orx_fixed(pool: &rayon_core::ThreadPool, data: &[u64], task_kind: TaskKind) -> Option<u64> {
    data.into_par()
        .runner(Runner::fixed_chunk(pool))
        .num_threads(NUM_THREADS)
        .map(|value| expensive_map(value, task_kind))
        .filter(selective_filter)
        .reduce(reduce_sum)
}

fn run_orx_dyn(pool: &rayon_core::ThreadPool, data: &[u64], task_kind: TaskKind) -> Option<u64> {
    data.into_par()
        .runner(Runner::dynamic_chunk(pool))
        .num_threads(NUM_THREADS)
        .map(|value| expensive_map(value, task_kind))
        .filter(selective_filter)
        .reduce(reduce_sum)
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

    let len = 1usize << args.len_exp;
    println!(
        "runner_mf_r example: task_kind={} len_exp={} len={} threads={}",
        args.task_kind.as_str(),
        args.len_exp,
        len,
        NUM_THREADS
    );

    let data = values(len);
    let pool = Pool::rayon(NUM_THREADS).expect("failed to build rayon thread pool");

    let (seq, _) = run_timed("seq", || run_seq(&data, args.task_kind));
    let (rayon, _) = run_timed("rayon", || run_rayon(&pool, &data, args.task_kind));
    let (orx_fixed, _) = run_timed("orx-fixed", || run_orx_fixed(&pool, &data, args.task_kind));
    let (orx_dyn, _) = run_timed("orx-dyn", || run_orx_dyn(&pool, &data, args.task_kind));

    assert_eq!(rayon, seq, "rayon output mismatch");
    assert_eq!(orx_fixed, seq, "orx-fixed output mismatch");
    assert_eq!(orx_dyn, seq, "orx-dyn output mismatch");

    println!("all methods produced identical outputs");
}
