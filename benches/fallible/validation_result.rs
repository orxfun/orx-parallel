//! Fallible validation benchmark with `Result` fail-fast semantics.
//! Simulates record decoding/validation with success-heavy, mixed, and
//! fail-early datasets to compare error-propagation overheads.

use criterion::{Criterion, criterion_group, criterion_main};
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
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "scenario"]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseErr {
    Missing,
    Invalid,
}

struct Exp;

fn should_be_invalid(idx: usize, len: usize, profile: InvalidProfile) -> bool {
    match profile {
        InvalidProfile::SuccessHeavy => false,
        InvalidProfile::Mixed => idx % 97 == 13,
        InvalidProfile::FailEarly => idx == 7 || idx == len / 2,
    }
}

fn inputs(len: usize, profile: InvalidProfile) -> Vec<String> {
    const SEED: u64 = 0xCAFE_BABE_1234_5678;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    (0..len)
        .map(|idx| {
            let qty: u32 = rng.random_range(1..=60);
            let unit: u32 = rng.random_range(100..=8_000);

            if should_be_invalid(idx, len, profile) {
                if idx % 2 == 0 {
                    format!("x,{}", unit)
                } else {
                    qty.to_string()
                }
            } else {
                format!("{},{}", qty, unit)
            }
        })
        .collect()
}

fn parse_line_total(row: &str) -> Result<u64, ParseErr> {
    let mut parts = row.split(',');

    let qty = parts
        .next()
        .ok_or(ParseErr::Missing)?
        .parse::<u64>()
        .map_err(|_| ParseErr::Invalid)?;

    let unit = parts
        .next()
        .ok_or(ParseErr::Missing)?
        .parse::<u64>()
        .map_err(|_| ParseErr::Invalid)?;

    Ok(qty * unit)
}

fn failure_kinds(input: &[String]) -> (bool, bool) {
    let mut has_missing = false;
    let mut has_invalid = false;

    for row in input {
        match parse_line_total(row) {
            Err(ParseErr::Missing) => has_missing = true,
            Err(ParseErr::Invalid) => has_invalid = true,
            Ok(_) => {}
        }
    }

    (has_missing, has_invalid)
}

fn seq_sum(input: &[String], threshold: u64) -> Result<u64, ParseErr> {
    let mut sum = 0_u64;
    for row in input {
        let total = parse_line_total(row)?;
        if total >= threshold {
            sum = sum.wrapping_add(total);
        }
    }
    Ok(sum)
}

fn rayon_sum(input: &[String], threshold: u64, num_threads: usize) -> Result<u64, ParseErr> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        input
            .into_par_iter()
            .map(|row| parse_line_total(row))
            .collect::<Result<Vec<_>, _>>()
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

    type Output = Result<u64, ParseErr>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len(), input_variant.profile)
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        const THRESHOLD: u64 = 2_000;

        match alg_variant {
            Method::Seq => seq_sum(input, THRESHOLD),
            Method::Rayon { nt } => rayon_sum(input, THRESHOLD, *nt),
            Method::Orx { nt } => input
                .as_slice()
                .into_par()
                .num_threads(*nt)
                .map(|row| parse_line_total(row))
                .into_fallible()
                .filter(|x| *x >= THRESHOLD)
                .sum(),
            Method::OrxFixed { nt } => input
                .as_slice()
                .into_par()
                .runner(Runner::fixed(Pool::once(*nt)))
                .num_threads(*nt)
                .map(|row| parse_line_total(row))
                .into_fallible()
                .filter(|x| *x >= THRESHOLD)
                .sum(),
        }
    }

    fn validate_output(&self, _: &Self::InputFactors, input: &Self::Input, output: &Self::Output) {
        let expected = seq_sum(input, 2_000);

        match expected {
            Ok(expected_sum) => {
                assert_eq!(*output, Ok(expected_sum));
            }
            Err(_) => {
                let (has_missing, has_invalid) = failure_kinds(input);

                match output {
                    Err(ParseErr::Missing) => {
                        assert!(
                            has_missing,
                            "output has Missing but input has no Missing failures"
                        );
                    }
                    Err(ParseErr::Invalid) => {
                        assert!(
                            has_invalid,
                            "output has Invalid but input has no Invalid failures"
                        );
                    }
                    Ok(sum) => {
                        panic!(
                            "expected failure for this input, but run produced success with sum={}",
                            sum
                        );
                    }
                }
            }
        }
    }
}

fn run(c: &mut Criterion) {
    let ns = [16, 20];
    let profiles = [
        InvalidProfile::SuccessHeavy,
        InvalidProfile::Mixed,
        InvalidProfile::FailEarly,
    ];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| profiles.map(|profile| InputVariant { n, profile }))
        .collect();

    let par_variants = |nt: usize| [Method::Rayon { nt }, Method::Orx { nt }, Method::OrxFixed { nt }];

    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "fallible_validation_result", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
