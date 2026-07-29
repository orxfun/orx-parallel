//! Contention-merge benchmark for grouped counting + top-k reduction.
//! Simulates key distributions from uniform to skewed to stress merge pressure
//! while comparing sequential and parallel aggregation strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const KEY_SPACE: usize = 4096;
const TOP_K: usize = 32;

#[derive(Debug, Clone, Copy)]
enum Dist {
    Uniform,
    Skewed,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    dist: Dist,
    num_threads: usize,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "dist", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.dist {
                Dist::Uniform => "uniform",
                Dist::Skewed => "skewed",
            }
            .to_string(),
            self.num_threads.to_string(),
        ]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
    Orx,
    OrxFixed,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::Seq => "seq",
                Self::Rayon => "rayon",
                Self::Orx => "orx",
                Self::OrxFixed => "orx-fixed",
            }
            .to_string(),
        ]
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

fn count_seq(input: &[u16]) -> Vec<u32> {
    let mut counts = vec![0_u32; KEY_SPACE];
    for key in input {
        counts[*key as usize] += 1;
    }
    counts
}

fn merge_counts(mut a: Vec<u32>, b: Vec<u32>) -> Vec<u32> {
    for (x, y) in a.iter_mut().zip(b) {
        *x += y;
    }
    a
}

fn count_rayon(input: &[u16], num_threads: usize) -> Vec<u32> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        input
            .into_par_iter()
            .fold(
                || vec![0_u32; KEY_SPACE],
                |mut local, key| {
                    local[*key as usize] += 1;
                    local
                },
            )
            .reduce(|| vec![0_u32; KEY_SPACE], merge_counts)
    })
}

fn count_orx(input: &[u16], num_threads: usize) -> Vec<u32> {
    let mut use_vec = UseVec::new(|_| vec![0_u32; KEY_SPACE]);

    input
        .into_par()
        .num_threads(num_threads)
        .use_vec(&mut use_vec)
        .for_each(|local, key| {
            local[*key as usize] += 1;
        });

    use_vec
        .into_vec()
        .into_iter()
        .reduce(merge_counts)
        .unwrap_or_else(|| vec![0_u32; KEY_SPACE])
}

fn count_orx_fixed(input: &[u16], num_threads: usize) -> Vec<u32> {
    let mut use_vec = UseVec::new(|_| vec![0_u32; KEY_SPACE]);

    input
        .into_par()
        .runner(Runner::fixed(Pool::default(num_threads)))
        .num_threads(num_threads)
        .use_vec(&mut use_vec)
        .for_each(|local, key| {
            local[*key as usize] += 1;
        });

    use_vec
        .into_vec()
        .into_iter()
        .reduce(merge_counts)
        .unwrap_or_else(|| vec![0_u32; KEY_SPACE])
}

fn topk_agg(counts: &[u32]) -> Agg {
    let mut entries: Vec<(usize, u32)> = counts.iter().copied().enumerate().collect();
    entries.sort_unstable_by(|(ka, ca), (kb, cb)| cb.cmp(ca).then(ka.cmp(kb)));

    let mut checksum = 0_u64;
    let mut top_sum = 0_u64;

    for (rank, (key, count)) in entries.into_iter().take(TOP_K).enumerate() {
        top_sum += count as u64;
        checksum ^= ((key as u64) << 20) ^ ((count as u64) << 7) ^ rank as u64;
    }

    Agg {
        total: counts.iter().map(|x| *x as u64).sum(),
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
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let counts = match alg_variant {
            Method::Seq => count_seq(input),
            Method::Rayon => count_rayon(input, input_variant.num_threads),
            Method::Orx => count_orx(input, input_variant.num_threads),
            Method::OrxFixed => count_orx_fixed(input, input_variant.num_threads),
        };

        topk_agg(&counts)
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(topk_agg(&count_seq(input)))
    }
}

fn run(c: &mut Criterion) {
    let num_threads_options = [4, 16];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant {
                    n: 16,
                    dist: Dist::Uniform,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    dist: Dist::Uniform,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    dist: Dist::Skewed,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    dist: Dist::Skewed,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "contention_merge_grouped_topk", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
