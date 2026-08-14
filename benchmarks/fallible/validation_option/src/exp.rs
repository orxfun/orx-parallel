use crate::alg::Method;
use crate::input::{InputVariant, InvalidProfile};
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

pub struct Exp;

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

fn sum_seq(input: &[String], threshold: u64) -> Option<u64> {
    let mut sum = 0_u64;
    for row in input {
        let total = parse_line_total(row)?;
        if total >= threshold {
            sum = sum.wrapping_add(total);
        }
    }
    Some(sum)
}

fn sum_rayon(input: &[String], threshold: u64) -> Option<u64> {
    input
        .par_iter()
        .map(|row| parse_line_total(row))
        .collect::<Option<Vec<_>>>()
        .map(|totals| {
            totals
                .into_iter()
                .filter(|x| *x >= threshold)
                .fold(0_u64, |acc, x| acc.wrapping_add(x))
        })
}

fn sum_orx(input: &[String], threshold: u64) -> Option<u64> {
    input
        .into_par()
        .map(|row| parse_line_total(row))
        .into_optional()
        .filter(|x| *x >= threshold)
        .sum()
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
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        const THRESHOLD: u64 = 2_000;

        match alg_variant {
            Method::Seq => sum_seq(input, THRESHOLD),
            Method::Rayon => sum_rayon(input, THRESHOLD),
            Method::OrxOnce => sum_orx(input, THRESHOLD),
            Method::OrxBasic => sum_orx(input, THRESHOLD),
            Method::OrxRayon => sum_orx(input, THRESHOLD),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(sum_seq(input, 2_000))
    }
}
