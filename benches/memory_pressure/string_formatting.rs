//! Memory pressure benchmark: large string formatting and allocation.
//! Simulates allocation-heavy workloads where output materialization dominates,
//! including buffer reuse and locality considerations.

mod utils;
use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::{pool::BasicPool, *};
use std::hint::black_box;
use std::sync::LazyLock;
use utils::Pool;

const NUM_WARMUPS_PER_POOL: usize = 5;

// Static BasicPools created once for the entire benchmark process, matching the
// --features persistent-pool scenario (Run B). Warmup runs at first access.
static BASIC_POOL_4: LazyLock<BasicPool> = LazyLock::new(|| {
    let pool = orx_parallel::Pool::basic(4);
    for _ in 0..NUM_WARMUPS_PER_POOL {
        (0..100_000_usize)
            .par()
            .pool(&pool)
            .map(|i| format_number(i as u64).0)
            .collect::<Vec<_>>();
    }
    pool
});

static BASIC_POOL_16: LazyLock<BasicPool> = LazyLock::new(|| {
    let pool = orx_parallel::Pool::basic(16);
    for _ in 0..NUM_WARMUPS_PER_POOL {
        (0..100_000_usize)
            .par()
            .pool(&pool)
            .map(|i| format_number(i as u64).0)
            .collect::<Vec<_>>();
    }
    pool
});

const CPU_MIX_ROUNDS: usize = 40;
fn cpu_mix(seed: u64) -> u64 {
    let mut x = black_box(seed ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

#[derive(Clone, Copy)]
struct InputVariant {
    size: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["size"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self.size {
                10_000 => "small-10k",
                100_000 => "medium-100k",
                _ => "unknown",
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
    OrxPersistent { nt: usize },
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
            Self::OrxPersistent { nt } => format!("orx-persist-{nt}"),
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringAgg {
    count: u64,
    total_len: u64,
    checksum: u64,
}

fn merge_agg(a: StringAgg, b: StringAgg) -> StringAgg {
    StringAgg {
        count: a.count + b.count,
        total_len: a.total_len + b.total_len,
        checksum: a.checksum + b.checksum,
    }
}

struct Exp;

fn format_number(idx: u64) -> (String, StringAgg) {
    let value = (idx.wrapping_mul(2654435761)).wrapping_add(0x9E3779B1);
    let formatted = format!("NUM_{:016x}_VAL_{}", idx, value);
    let len = formatted.len() as u64;
    let checksum = cpu_mix((idx ^ value).wrapping_mul(31).wrapping_add(len));

    (
        formatted,
        StringAgg {
            count: 1,
            total_len: len,
            checksum,
        },
    )
}

fn seq_format_and_collect(n: usize) -> (Vec<String>, StringAgg) {
    let mut strings = Vec::with_capacity(n);
    let mut agg = StringAgg::default();

    for i in 0..n {
        let (s, a) = format_number(i as u64);
        agg = merge_agg(agg, a);
        strings.push(s);
    }

    (strings, agg)
}

fn rayon_format_and_collect(n: usize, pool: &mut rayon::ThreadPool) -> (Vec<String>, StringAgg) {
    use rayon::prelude::*;

    pool.install(|| {
        let pairs: Vec<_> = (0..n)
            .into_par_iter()
            .map(|i| format_number(i as u64))
            .collect();

        let strings = pairs.iter().map(|(s, _)| s.clone()).collect();
        let mut agg = StringAgg::default();
        for (_, a) in &pairs {
            agg = merge_agg(agg, *a);
        }

        (strings, agg)
    })
}

fn orx_format_and_collect(n: usize, pool: impl ParThreadPool) -> (Vec<String>, StringAgg) {
    let nt: usize = pool.max_num_threads().into();
    let mut stats = vec![StringAgg::default(); nt];

    let strings = (0..n)
        .par()
        .pool(pool)
        .use_slice(&mut stats)
        .map(|stats, i| {
            let (s, agg) = format_number(i as u64);
            *stats = merge_agg(*stats, agg);
            s
        })
        .collect();
    let agg = stats.into_iter().reduce(merge_agg).unwrap_or_default();

    (strings, agg)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Output {
    strings: Vec<String>,
    agg: StringAgg,
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = usize;

    type Output = Output;

    type GroupArtifact = Pool;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        input_variant.size
    }

    fn group_artifact(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::GroupArtifact {
        match alg_variant {
            Method::Seq => Pool::Seq,
            Method::Rayon { nt } => {
                use rayon::prelude::*;
                let mut pool = Pool::new_rayon(*nt);
                for _ in 0..NUM_WARMUPS_PER_POOL {
                    pool.rayon().install(|| {
                        (0..*input)
                            .into_par_iter()
                            .map(|i| format_number(i as u64).0)
                            .collect::<Vec<_>>()
                    });
                }
                pool
            }
            Method::Orx { nt } => {
                let mut pool = Pool::new_once(*nt);
                for _ in 0..NUM_WARMUPS_PER_POOL {
                    (0..*input)
                        .par()
                        .pool(pool.once())
                        .map(|i| format_number(i as u64).0)
                        .collect::<Vec<_>>();
                }
                pool
            }
            Method::OrxPersistent { nt } => {
                // let mut pool = Pool::new_basic(*nt);
                // for _ in 0..NUM_WARMUPS_PER_POOL {
                //     (0..*input)
                //         .par()
                //         .pool(pool.basic())
                //         .map(|i| format_number(i as u64).0)
                //         .collect::<Vec<_>>();
                // }
                // pool

                // Touch the static to trigger lazy init + warmup (no-op on subsequent calls).
                // The actual pool lives for the entire benchmark process — no per-group creation.
                let _ = match nt {
                    4 => &*BASIC_POOL_4,
                    16 => &*BASIC_POOL_16,
                    _ => panic!("unsupported nt for orx-persist static pool"),
                };
                Pool::Seq
            }
        }
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
        pool: &mut Self::GroupArtifact,
    ) -> Self::Output {
        let (strings, agg) = match alg_variant {
            Method::Seq => seq_format_and_collect(*input),
            Method::Rayon { nt: _ } => rayon_format_and_collect(*input, pool.rayon()),
            Method::Orx { nt: _ } => orx_format_and_collect(*input, pool.once()),
            // Method::OrxPersistent { nt: _ } => orx_format_and_collect(*input, pool.basic()),
            Method::OrxPersistent { nt } => {
                let pool: &BasicPool = match nt {
                    4 => &BASIC_POOL_4,
                    16 => &BASIC_POOL_16,
                    _ => panic!("unsupported nt for orx-persist static pool"),
                };
                orx_format_and_collect(*input, pool)
            }
        };

        Output { strings, agg }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let (strings, agg) = seq_format_and_collect(*input);
        Some(Output { strings, agg })
    }
}

fn run(c: &mut Criterion) {
    let treatments: Vec<_> = [10_000, 100_000]
        .into_iter()
        .map(|size| InputVariant { size })
        .collect();

    let par_variants = |nt: usize| {
        [
            // Method::Rayon { nt },
            // Method::Orx { nt },
            Method::OrxPersistent { nt },
        ]
    };

    let mut variants = vec![Method::Seq];
    // variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(
        c,
        "memory_pressure_string_formatting",
        &treatments,
        &variants,
    );
}

criterion_group!(benches, run);
criterion_main!(benches);
