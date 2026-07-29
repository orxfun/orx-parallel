//! Fallible validation benchmark with `Option` fail-fast semantics.
//! Simulates record decoding/validation with success-heavy, mixed, and
//! fail-early datasets to compare missing-value propagation costs.

use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Clone, Copy)]
enum InvalidProfile {
    SuccessHeavy,
    Mixed,
    FailEarly,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    profile: InvalidProfile,
    num_threads: usize,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "scenario", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.profile {
                InvalidProfile::SuccessHeavy => "success-heavy",
                InvalidProfile::Mixed => "mixed",
                InvalidProfile::FailEarly => "fail-early",
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

struct Exp;

fn should_be_invalid(idx: usize, len: usize, profile: InvalidProfile) -> bool {
    match profile {
        InvalidProfile::SuccessHeavy => false,
        InvalidProfile::Mixed => idx % 89 == 11,
        InvalidProfile::FailEarly => idx == 5 || idx == len / 2,
    }
}

fn inputs(len: usize, profile: InvalidProfile) -> Vec<String> {
    const SEED: u64 = 0x1234_9999_ABCD_F00D;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    (0..len)
        .map(|idx| {
            let qty: u32 = rng.random_range(1..=60);
            let unit: u32 = rng.random_range(100..=8_000);

            if should_be_invalid(idx, len, profile) {
                if idx % 2 == 0 {
                    format!("bad,{}", unit)
                } else {
                    qty.to_string()
                }
            } else {
                format!("{},{}", qty, unit)
            }
        })
        .collect()
}

fn parse_line_total(row: &str) -> Option<u64> {
    let mut parts = row.split(',');

    let qty = parts.next()?.parse::<u64>().ok()?;
    let unit = parts.next()?.parse::<u64>().ok()?;

    Some(qty * unit)
}

fn seq_sum(input: &[String], threshold: u64) -> Option<u64> {
    let mut sum = 0_u64;
    for row in input {
        let total = parse_line_total(row)?;
        if total >= threshold {
            sum = sum.wrapping_add(total);
        }
    }
    Some(sum)
}

fn rayon_sum(input: &[String], threshold: u64, num_threads: usize) -> Option<u64> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        input
            .into_par_iter()
            .map(|row| parse_line_total(row))
            .collect::<Option<Vec<_>>>()
            .map(|totals| {
                totals
                    .into_iter()
                    .filter(|x| *x >= threshold)
                    .fold(0_u64, |acc, x| acc.wrapping_add(x))
            })
    })
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<String>;

    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len(), input_variant.profile)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        const THRESHOLD: u64 = 2_000;

        match alg_variant {
            Method::Seq => seq_sum(input, THRESHOLD),
            Method::Rayon => rayon_sum(input, THRESHOLD, input_variant.num_threads),
            Method::Orx => input
                .as_slice()
                .into_par()
                .num_threads(input_variant.num_threads)
                .map(|row| parse_line_total(row))
                .into_optional()
                .filter(|x| *x >= THRESHOLD)
                .sum(),
            Method::OrxFixed => input
                .as_slice()
                .into_par()
                .runner(Runner::fixed(Pool::default(input_variant.num_threads)))
                .num_threads(input_variant.num_threads)
                .map(|row| parse_line_total(row))
                .into_optional()
                .filter(|x| *x >= THRESHOLD)
                .sum(),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(seq_sum(input, 2_000))
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
                    profile: InvalidProfile::SuccessHeavy,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    profile: InvalidProfile::SuccessHeavy,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    profile: InvalidProfile::Mixed,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    profile: InvalidProfile::Mixed,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    profile: InvalidProfile::FailEarly,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    profile: InvalidProfile::FailEarly,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "fallible_validation_option", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
