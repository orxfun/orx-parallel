use crate::alg::Method;
use crate::input::{InputVariant, InvalidProfile};
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseErr {
    Missing,
    Invalid,
}

pub struct Exp;

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

fn sum_seq(input: &[String], threshold: u64) -> Result<u64, ParseErr> {
    let mut sum = 0_u64;
    for row in input {
        let total = parse_line_total(row)?;
        if total >= threshold {
            sum = sum.wrapping_add(total);
        }
    }
    Ok(sum)
}

fn sum_rayon(input: &[String], threshold: u64) -> Result<u64, ParseErr> {
    input
        .par_iter()
        .map(|row| parse_line_total(row))
        .collect::<Result<Vec<_>, _>>()
        .map(|totals| {
            totals
                .into_iter()
                .filter(|x| *x >= threshold)
                .fold(0_u64, |acc, x| acc.wrapping_add(x))
        })
}

fn sum_orx(input: &[String], threshold: u64) -> Result<u64, ParseErr> {
    input
        .into_par()
        .map(|row| parse_line_total(row))
        .into_fallible()
        .filter(|x| *x >= threshold)
        .sum()
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
            Method::Seq => sum_seq(input, THRESHOLD),
            Method::Rayon => sum_rayon(input, THRESHOLD),
            Method::OrxOnce => sum_orx(input, THRESHOLD),
            Method::OrxBasic => sum_orx(input, THRESHOLD),
            Method::OrxRayon => sum_orx(input, THRESHOLD),
        }
    }

    fn validate_output(&self, _: &Self::InputFactors, input: &Self::Input, output: &Self::Output) {
        let expected = sum_seq(input, 2_000);

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
