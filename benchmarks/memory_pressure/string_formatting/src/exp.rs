use crate::{alg::Method, input::InputVariant};
use orx_criterion::Experiment;
use std::hint::black_box;

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = usize;

    type Output = Output;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        input_variant.size
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (strings, agg) = match alg_variant {
            Method::Seq => run_seq(*input),
            Method::Rayon => run_rayon(*input),
            Method::OrxOnce => run_orx(*input),
            Method::OrxBasic => run_orx(*input),
            Method::OrxRayon => run_orx(*input),
        };

        Output { strings, agg }
    }
}

// computation helpers

const CPU_MIX_ROUNDS: usize = 40;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Output {
    strings: Vec<String>,
    agg: StringAgg,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringAgg {
    count: u64,
    total_len: u64,
    checksum: u64,
}

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

fn merge_agg(a: StringAgg, b: StringAgg) -> StringAgg {
    StringAgg {
        count: a.count + b.count,
        total_len: a.total_len + b.total_len,
        checksum: a.checksum + b.checksum,
    }
}

// computation variants

fn run_seq(n: usize) -> (Vec<String>, StringAgg) {
    let mut strings = Vec::with_capacity(n);
    let mut agg = StringAgg::default();

    for i in 0..n {
        let (s, a) = format_number(i as u64);
        agg = merge_agg(agg, a);
        strings.push(s);
    }

    (strings, agg)
}

fn run_rayon(n: usize) -> (Vec<String>, StringAgg) {
    use rayon::prelude::*;

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
}

fn run_orx(n: usize) -> (Vec<String>, StringAgg) {
    use orx_parallel::*;
    let mut stats = vec![StringAgg::default(); 32];

    let strings = (0..n)
        .par()
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
