use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::hint::black_box;

const FEW_N: usize = 1 << 12;
const MANY_N: usize = 1 << 18;

const NORMAL_ITERS_LIGHT: u32 = 50;
const NORMAL_ITERS_HEAVY: u32 = 500;
const OUTLIER_MULTIPLIER_MIN: u32 = 10;
const OUTLIER_MULTIPLIER_MAX: u32 = 100;

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    heavy: bool,
    heterogeneity_percent: u8,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "heavy", "het"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.n.to_string(),
            if self.heavy { "true" } else { "false" }.to_string(),
            format!("{}%", self.heterogeneity_percent),
        ]
    }
}

#[derive(Clone, Copy)]
struct WorkItem {
    seed: u64,
    iterations: u32,
}

#[derive(Debug)]
enum Method {
    Seq,
    Rayon { nt: usize },
    OrxFixed { nt: usize, chunk_size: usize },
    Orx { nt: usize, chunk_size: usize },
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon { nt } => format!("rayon-{nt}"),
            Self::OrxFixed { nt, chunk_size } => format!("orx-fixed-auto-{nt}-{chunk_size}"),
            Self::Orx { nt, chunk_size } => format!("orx-{nt}-{chunk_size}"),
        }]
    }
}

fn seed_for_input(variant: &InputVariant) -> u64 {
    let n_flag = if variant.n == FEW_N { 0xA1 } else { 0xB2 };
    let heavy_flag = if variant.heavy { 0xC3 } else { 0xD4 };
    let het = variant.heterogeneity_percent as u64;
    0xC0FF_EE12_3456_7890u64 ^ ((n_flag as u64) << 48) ^ ((heavy_flag as u64) << 40) ^ (het << 8)
}

fn build_workload(variant: &InputVariant) -> Vec<WorkItem> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed_for_input(variant));
    let mut items = vec![
        WorkItem {
            seed: 0,
            iterations: match variant.heavy {
                true => NORMAL_ITERS_HEAVY,
                false => NORMAL_ITERS_LIGHT,
            }
        };
        variant.n
    ];

    for (idx, item) in items.iter_mut().enumerate() {
        item.seed = idx as u64 ^ rng.random::<u64>();
    }

    let outlier_count = (variant.n * variant.heterogeneity_percent as usize) / 100;
    if outlier_count == 0 {
        return items;
    }

    let mut indices: Vec<usize> = (0..variant.n).collect();
    indices.shuffle(&mut rng);

    for idx in indices.into_iter().take(outlier_count) {
        let m = rng.random_range(OUTLIER_MULTIPLIER_MIN..=OUTLIER_MULTIPLIER_MAX);
        let iters = items[idx].iterations.saturating_mul(m);
        items[idx].iterations = iters;
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

struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<WorkItem>;
    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        build_workload(input_variant)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => self.expected_output(input_variant, input).unwrap(),
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(*nt)
                    .build()
                    .unwrap();
                pool.install(|| input.par_iter().map(do_work).max())
            }
            Method::OrxFixed { nt, chunk_size } => input
                .par()
                .chunk_size(*chunk_size)
                .runner(Runner::fixed(Pool::once(*nt)))
                .num_threads(*nt)
                .map(do_work)
                .max(),
            Method::Orx { nt, chunk_size } => input
                .par()
                .chunk_size(*chunk_size)
                .runner(Runner::adaptive(Pool::once(*nt)))
                .num_threads(*nt)
                .map(do_work)
                .max(),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(input.iter().map(do_work).max())
    }
}

fn run(c: &mut Criterion) {
    let sizes = [FEW_N, MANY_N];
    let heavy_options = [false, true];
    let het_options = [2, 5, 10];

    let mut treatments = Vec::new();
    for &n in &sizes {
        for &heavy in &heavy_options {
            for &heterogeneity_percent in &het_options {
                treatments.push(InputVariant {
                    n,
                    heavy,
                    heterogeneity_percent,
                });
            }
        }
    }

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::Orx { nt, chunk_size: 0 },
            Method::Orx { nt, chunk_size: 1 },
            Method::OrxFixed { nt, chunk_size: 0 },
            Method::OrxFixed { nt, chunk_size: 1 },
        ]
    };

    let mut variants = vec![Method::Seq];
    for nt in [1, 4, 16] {
        variants.extend(par_variants(nt));
    }

    Exp.bench(c, "het_advanced", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
