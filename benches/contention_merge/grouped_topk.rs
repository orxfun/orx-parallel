//! Contention-merge benchmark for grouped counting + top-k reduction.
//! Simulates key distributions from uniform to skewed to stress merge pressure
//! while comparing sequential and parallel aggregation strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

const KEY_SPACE: usize = 4096;
const TOP_K: usize = 32;

const CPU_MIX_ROUNDS: usize = 40;
fn cpu_mix(x: u16) -> u64 {
    let mut x = black_box(x as u64);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

#[derive(Debug, Clone, Copy)]
enum Dist {
    Uniform,
    Skewed,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    dist: Dist,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "dist"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.dist {
                Dist::Uniform => "uniform",
                Dist::Skewed => "skewed",
            }
            .to_string(),
        ]
    }
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Agg {
    total: u64,
    top_sum: u64,
    checksum: u64,
}

struct Exp;

fn inputs(len: usize, dist: Dist) -> Vec<u16> {
    const SEED: u64 = 0xFACE_CAFE_1357_2468;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    (0..len)
        .map(|_| {
            let key = match dist {
                Dist::Uniform => rng.random_range(0..KEY_SPACE as u64),
                Dist::Skewed => {
                    let u = rng.random::<f64>();
                    let x = (u * u * u * u) * KEY_SPACE as f64;
                    x as u64
                }
            }
            .min((KEY_SPACE - 1) as u64);
            key as u16
        })
        .collect()
}

fn count_seq(input: &[u16]) -> Vec<u64> {
    let mut counts = vec![0u64; KEY_SPACE];
    for key in input {
        counts[*key as usize] += cpu_mix(*key);
    }
    counts
}

fn merge_counts(mut a: Vec<u64>, b: Vec<u64>) -> Vec<u64> {
    for (x, y) in a.iter_mut().zip(b) {
        *x += y;
    }
    a
}

fn count_rayon(input: &[u16], num_threads: usize) -> Vec<u64> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        input
            .into_par_iter()
            .fold(
                || vec![0u64; KEY_SPACE],
                |mut local, key| {
                    local[*key as usize] += cpu_mix(*key);
                    local
                },
            )
            .reduce(|| vec![0u64; KEY_SPACE], merge_counts)
    })
}

fn count_orx(input: &[u16], fixed_runner: bool, num_threads: usize) -> Vec<u64> {
    let mut use_vec = UseVec::new(|_| vec![0u64; KEY_SPACE]);

    let par = input
        .into_par()
        .num_threads(num_threads)
        .use_vec(&mut use_vec);

    match fixed_runner {
        false => par.for_each(|local, key| {
            local[*key as usize] += cpu_mix(*key);
        }),
        true => par
            .runner(Runner::fixed())
            .num_threads(num_threads)
            .for_each(|local, key| {
                local[*key as usize] += cpu_mix(*key);
            }),
    }

    use_vec
        .into_vec()
        .into_iter()
        .reduce(merge_counts)
        .unwrap_or_else(|| vec![0u64; KEY_SPACE])
}

fn topk_agg(counts: &[u64]) -> Agg {
    let mut entries: Vec<(usize, u64)> = counts.iter().copied().enumerate().collect();
    entries.sort_unstable_by(|(ka, ca), (kb, cb)| cb.cmp(ca).then(ka.cmp(kb)));

    let mut checksum = 0_u64;
    let mut top_sum = 0_u64;

    for (rank, (key, count)) in entries.into_iter().take(TOP_K).enumerate() {
        top_sum += count;
        checksum ^= ((key as u64) << 20) ^ (count << 7) ^ rank as u64;
    }

    Agg {
        total: counts.iter().sum(),
        top_sum,
        checksum,
    }
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<u16>;

    type Output = Agg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len(), input_variant.dist)
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let counts = match alg_variant {
            Method::Seq => count_seq(input),
            Method::Rayon { nt } => count_rayon(input, *nt),
            Method::Orx { nt } => count_orx(input, false, *nt),
            Method::OrxFixed { nt } => count_orx(input, true, *nt),
        };

        topk_agg(&counts)
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(topk_agg(&count_seq(input)))
    }
}

fn run(c: &mut Criterion) {
    let ns = [16, 20];
    let distributions = [Dist::Uniform, Dist::Skewed];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| distributions.map(|dist| InputVariant { n, dist }))
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

    Exp.bench(c, "contention_merge_grouped_topk", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
