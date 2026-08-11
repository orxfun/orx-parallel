#![allow(dead_code)]

//! Memory pressure benchmark: large string formatting and allocation.
//! Simulates allocation-heavy workloads where output materialization dominates,
//! including buffer reuse and locality considerations.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rayon::ThreadPoolBuilder;
use std::hint::black_box;

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
struct StringAgg {
    count: u64,
    total_len: u64,
    checksum: u64,
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

fn seq_format_and_collect(n: usize) -> Vec<String> {
    let mut strings = Vec::with_capacity(n);

    for i in 0..n {
        let (s, _) = format_number(i as u64);
        strings.push(s);
    }

    strings
}

fn rayon_format_and_collect(n: usize, num_threads: usize) -> Vec<String> {
    use rayon::prelude::*;

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        (0..n)
            .into_par_iter()
            .map(|i| format_number(i as u64).0)
            .collect()
    })
}

fn orx_format_and_collect(n: usize, num_threads: usize) -> Vec<String> {
    (0..n)
        .par()
        .num_threads(num_threads)
        .map(|i| format_number(i as u64).0)
        .collect()
}

fn orx_fixed_format_and_collect(n: usize, num_threads: usize) -> Vec<String> {
    (0..n)
        .par()
        .runner(Runner::fixed())
        .num_threads(num_threads)
        .map(|i| format_number(i as u64).0)
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Output {
    strings: Vec<String>,
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = usize;

    type Output = Output;

    type GroupArtifact = ();

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        input_variant.size
    }

    fn group_artifact(
        &mut self,
        _: &Self::InputFactors,
        _: &Self::AlgFactors,
        _: &Self::Input,
    ) -> Self::GroupArtifact {
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
        _: &mut Self::GroupArtifact,
    ) -> Self::Output {
        let strings = match alg_variant {
            Method::Seq => seq_format_and_collect(*input),
            Method::Rayon { nt } => rayon_format_and_collect(*input, *nt),
            Method::Orx { nt } => orx_format_and_collect(*input, *nt),
            Method::OrxFixed { nt } => orx_fixed_format_and_collect(*input, *nt),
        };

        Output { strings }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let strings = seq_format_and_collect(*input);
        Some(Output { strings })
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
            Method::Orx { nt },
            // Method::OrxFixed { nt },
        ]
    };

    let mut variants = vec![Method::Seq];
    // variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));
    variants.extend(par_variants(32));

    Exp.bench(
        c,
        "memory_pressure_string_formatting4",
        &treatments,
        &variants,
    );
}

criterion_group!(benches, run);
criterion_main!(benches);
