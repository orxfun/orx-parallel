use clap::Parser;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::hint::black_box;
use std::time::Instant;

const NUM_THREADS: usize = 16;
const HOMOGENEOUS_WORK: usize = 96;
const HETEROGENEOUS_LIGHT_WORK: usize = 1;
const HETEROGENEOUS_MEDIUM_WORK: usize = 20;
const HETEROGENEOUS_HEAVY_WORK: usize = 2200;
const HETEROGENEOUS_EXTREME_WORK: usize = 66200;

#[derive(Clone, Copy)]
struct WorkItem {
    idx: usize,
    seed: u64,
}

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

    /// Number of warmup runs per method before measured timings.
    #[arg(long, default_value_t = 1)]
    warmup_runs: usize,

    /// Set to true to activate diagnostics printing with orx variants.
    #[arg(long, default_value_t = false)]
    diagnostics: bool,
}

fn values(len: usize) -> Vec<WorkItem> {
    const SEED: u64 = 0xD1CE_BA5E;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    let region_len = (len / 24).max(64);

    (0..len)
        .map(|idx| {
            let region = idx / region_len;
            let jitter = rng.random::<u64>();
            let seed = jitter
                ^ ((idx as u64 + 1) * 0x9E37_79B9_7F4A_7C15)
                ^ ((region as u64 + 11) * 0xA076_1D64_78BD_642F);
            WorkItem { idx, seed }
        })
        .collect()
}

fn work_units(item: &WorkItem, task_kind: TaskKind, len: usize) -> usize {
    match task_kind {
        TaskKind::Homogeneous => HOMOGENEOUS_WORK,
        TaskKind::Heterogeneous => {
            let region_len = (len / 24).max(64);
            let region = item.idx / region_len;
            let offset_in_region = item.idx % region_len;

            // Region classes intentionally create hard scheduling cases:
            // - clustered regions: long tasks come in contiguous blocks
            // - bursty regions: long tasks come periodically in mini-waves
            // - sparse regions: long tasks are scattered as outliers
            let region_class = ((region as u64).wrapping_mul(37) ^ (item.seed >> 19)) % 7;

            let clustered = match region_class {
                0 | 1 => {
                    let left = region_len / 5;
                    let right = region_len - (region_len / 6);
                    offset_in_region >= left && offset_in_region <= right
                }
                _ => false,
            };

            let bursty = match region_class {
                2 | 3 => {
                    let burst_window = region_len / 12 + 3;
                    let phase = ((item.seed as usize) / 97) % (burst_window * 3);
                    offset_in_region % (burst_window * 3) >= phase
                        && offset_in_region % (burst_window * 3) < phase + burst_window
                }
                _ => false,
            };

            let sparse_outlier = match region_class {
                4 | 5 => ((item.seed ^ item.idx as u64) & 0x7f) == 0,
                _ => false,
            };

            match (clustered, bursty, sparse_outlier) {
                (true, _, _) => HETEROGENEOUS_EXTREME_WORK,
                (_, true, _) => HETEROGENEOUS_HEAVY_WORK,
                (_, _, true) => HETEROGENEOUS_EXTREME_WORK,
                _ => {
                    let noisy_medium = ((item.seed >> 11) + item.idx as u64).is_multiple_of(9);
                    if noisy_medium {
                        HETEROGENEOUS_MEDIUM_WORK
                    } else {
                        HETEROGENEOUS_LIGHT_WORK
                    }
                }
            }
        }
    }
}

fn expensive_map(item: &WorkItem, task_kind: TaskKind, len: usize) -> u64 {
    let mut acc = black_box(item.seed ^ 0xA076_1D64_78BD_642F);
    let rounds = work_units(item, task_kind, len);

    for round in 0..rounds {
        let salt = black_box((round as u64 + 1) * 0xE703_7ED1_A0B4_28DB ^ item.idx as u64);
        acc = acc.rotate_left(11) ^ salt;
        acc = acc.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        acc ^= acc >> 29;
    }

    acc ^ item.seed.rotate_left(7)
}

fn selective_filter(value: &u64) -> bool {
    let folded = value ^ value.rotate_right(17);
    folded.count_ones() % 3 != 0
}

fn reduce_sum(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

fn run_seq(data: &[WorkItem], task_kind: TaskKind) -> Option<u64> {
    let len = data.len();
    data.iter()
        .map(|value| expensive_map(value, task_kind, len))
        .filter(selective_filter)
        .reduce(reduce_sum)
}

fn run_rayon(pool: &rayon_core::ThreadPool, data: &[WorkItem], task_kind: TaskKind) -> Option<u64> {
    let len = data.len();
    pool.install(|| {
        data.par_iter()
            .map(|value| expensive_map(value, task_kind, len))
            .filter(selective_filter)
            .reduce_with(reduce_sum)
    })
}

fn run_orx_fixed(
    pool: &rayon_core::ThreadPool,
    data: &[WorkItem],
    task_kind: TaskKind,
    diagnostics: bool,
) -> Option<u64> {
    let len = data.len();
    let par = data
        .into_par()
        .runner(Runner::fixed_chunk(pool))
        .num_threads(NUM_THREADS)
        .map(|value| expensive_map(value, task_kind, len))
        .filter(selective_filter);
    match diagnostics {
        true => par.runner_with_diagnostics().reduce(reduce_sum),
        false => par.reduce(reduce_sum),
    }
}

#[cfg(feature = "experimental")]
fn run_orx_dyn(
    pool: &rayon_core::ThreadPool,
    data: &[WorkItem],
    task_kind: TaskKind,
    diagnostics: bool,
) -> Option<u64> {
    let len = data.len();
    let par = data
        .into_par()
        .runner(Runner::dynamic_chunk(pool))
        .num_threads(NUM_THREADS)
        .map(|value| expensive_map(value, task_kind, len))
        .filter(selective_filter);
    match diagnostics {
        true => par.runner_with_diagnostics().reduce(reduce_sum),
        false => par.reduce(reduce_sum),
    }
}

fn run_timed(name: &str, f: impl FnOnce() -> Option<u64>) -> (Option<u64>, f64) {
    let start = Instant::now();
    let out = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
    println!("{name:<10} | {elapsed_ms:>9.3} ms");
    (out, elapsed_ms)
}

fn run_warmup(mut f: impl FnMut() -> Option<u64>, warmup_runs: usize) {
    for _ in 0..warmup_runs {
        let _ = f();
    }
}

fn main() {
    let args = Args::parse();
    assert!(args.len_exp < usize::BITS as usize, "len_exp is too large");

    let len = 1usize << args.len_exp;
    println!(
        "runner_mf_r example: task_kind={} len_exp={} len={} threads={} warmup_runs={}",
        args.task_kind.as_str(),
        args.len_exp,
        len,
        NUM_THREADS,
        args.warmup_runs
    );

    let data = values(len);
    let pool = Pool::rayon(NUM_THREADS).expect("failed to build rayon thread pool");

    run_warmup(|| run_seq(&data, args.task_kind), args.warmup_runs);
    run_warmup(|| run_rayon(&pool, &data, args.task_kind), args.warmup_runs);
    run_warmup(
        || run_orx_fixed(&pool, &data, args.task_kind, false),
        args.warmup_runs,
    );
    #[cfg(feature = "experimental")]
    run_warmup(
        || run_orx_dyn(&pool, &data, args.task_kind, false),
        args.warmup_runs,
    );

    let (seq, _) = run_timed("seq", || run_seq(&data, args.task_kind));
    let (rayon, _) = run_timed("rayon", || run_rayon(&pool, &data, args.task_kind));
    let (orx_fixed, _) = run_timed("orx-fixed", || {
        run_orx_fixed(&pool, &data, args.task_kind, args.diagnostics)
    });
    #[cfg(feature = "experimental")]
    let (orx_dyn, _) = run_timed("orx-dyn", || {
        run_orx_dyn(&pool, &data, args.task_kind, args.diagnostics)
    });

    assert_eq!(rayon, seq, "rayon output mismatch");
    assert_eq!(orx_fixed, seq, "orx-fixed output mismatch");
    #[cfg(feature = "experimental")]
    assert_eq!(orx_dyn, seq, "orx-dyn output mismatch");

    println!("all methods produced identical outputs");
}
