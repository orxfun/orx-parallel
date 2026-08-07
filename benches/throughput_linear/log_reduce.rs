//! Throughput-linear log processing benchmark for map/filter/reduce workloads.
//! Simulates parsing event records, filtering actionable entries, and reducing
//! them into aggregate counters/checksums across execution strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    heavy: bool,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "task"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.heavy {
                true => "heavy",
                false => "light",
            }
            .to_string(),
        ]
    }
}

#[derive(Debug)]
enum Method {
    Seq,
    Rayon { nt: usize },
    Orx { nt: usize },
    OrxFixed { nt: usize },
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon { nt } => format!("rayon-{nt}"),
            Self::Orx { nt } => format!("orx-{nt}"),
            Self::OrxFixed { nt } => format!("orx-fixed-{nt}"),
        }]
    }
}

#[derive(Clone, Copy, Debug)]
struct LogRecord {
    severity: u8,
    code: u16,
    user_id: u32,
    ts: u64,
    payload_len: u16,
    payload_seed: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Agg {
    err: u64,
    warn: u64,
    info: u64,
    checksum: u64,
}

struct Exp;

fn inputs(len: usize) -> Vec<LogRecord> {
    const SEED: u64 = 0x55AA_A1A1_9090_F0F0;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    (0..len)
        .map(|idx| LogRecord {
            severity: rng.random_range(0..=5),
            code: rng.random_range(100..=899),
            user_id: rng.random_range(1..=200_000),
            ts: 1_700_000_000 + idx as u64,
            payload_len: rng.random_range(32..=1536),
            payload_seed: rng.random::<u64>() ^ (idx as u64).rotate_right(9),
        })
        .collect()
}

fn cpu_mix(seed: u64, rounds: usize) -> u64 {
    let mut x = black_box(seed ^ 0xA076_1D64_78BD_642F);
    for r in 0..rounds {
        let salt = black_box((r as u64 + 3) * 0x9E37_79B9_7F4A_7C15);
        x ^= salt;
        x = x.rotate_left(7).wrapping_mul(0xD134_2543_DE82_EEF9);
        x ^= x >> 31;
    }
    x
}

fn keep(record: &LogRecord) -> bool {
    (record.severity >= 3 && !record.code.is_multiple_of(7)) || record.code == 777
}

fn to_bucket(record: &LogRecord, heavy: bool) -> Agg {
    let rounds = if heavy {
        8 + (record.payload_len as usize / 96)
    } else {
        2
    };

    let parsed = cpu_mix(
        record.payload_seed ^ record.ts ^ ((record.user_id as u64) << 17),
        rounds,
    );

    let mut agg = Agg {
        checksum: parsed.wrapping_add(record.code as u64),
        ..Agg::default()
    };

    match record.severity {
        4 | 5 => agg.err = 1,
        3 => agg.warn = 1,
        _ => agg.info = 1,
    }

    agg
}

fn merge(a: Agg, b: Agg) -> Agg {
    Agg {
        err: a.err + b.err,
        warn: a.warn + b.warn,
        info: a.info + b.info,
        checksum: a.checksum.wrapping_add(b.checksum),
    }
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<LogRecord>;

    type Output = Option<Agg>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len())
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let h = input_variant.heavy;
        let input = input.as_slice();

        match alg_variant {
            Method::Seq => input
                .iter()
                .filter(|r| keep(r))
                .map(|r| to_bucket(r, h))
                .reduce(merge),
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| {
                    input
                        .into_par_iter()
                        .filter(|r| keep(r))
                        .map(|r| to_bucket(r, h))
                        .reduce_with(merge)
                })
            }
            Method::Orx { nt } => input
                .into_par()
                .num_threads(*nt)
                .filter(|r| keep(r))
                .map(|r| to_bucket(r, h))
                .reduce(merge),
            Method::OrxFixed { nt } => input
                .into_par()
                .runner(Runner::fixed())
                .num_threads(*nt)
                .filter(|r| keep(r))
                .map(|r| to_bucket(r, h))
                .reduce(merge),
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        let h = input_variant.heavy;
        Some(
            input
                .iter()
                .filter(|r| keep(r))
                .map(|r| to_bucket(r, h))
                .reduce(merge),
        )
    }
}

fn run(c: &mut Criterion) {
    let ns = [16, 20];
    let heavy_options = [false, true];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| heavy_options.map(|heavy| InputVariant { n, heavy }))
        .collect();

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::Orx { nt },
            Method::OrxFixed { nt },
        ]
    };

    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "throughput_linear_log_reduce", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
