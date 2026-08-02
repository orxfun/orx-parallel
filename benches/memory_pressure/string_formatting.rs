//! Memory pressure benchmark: large string formatting and allocation.
//! Simulates allocation-heavy workloads where output materialization dominates,
//! including buffer reuse and locality considerations.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rayon::ThreadPoolBuilder;

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
                1_000_000 => "small-1m",
                10_000_000 => "medium-10m",
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

fn merge_agg(a: StringAgg, b: StringAgg) -> StringAgg {
    StringAgg {
        count: a.count + b.count,
        total_len: a.total_len + b.total_len,
        checksum: a.checksum ^ b.checksum,
    }
}

struct Exp;

fn format_number(idx: u64) -> (String, StringAgg) {
    let value = (idx.wrapping_mul(2654435761)).wrapping_add(0x9E3779B1);
    let formatted = format!("NUM_{:016x}_VAL_{}", idx, value);
    let len = formatted.len() as u64;
    let checksum = (idx ^ value).wrapping_mul(31).wrapping_add(len);

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

fn rayon_format_and_collect(n: usize, num_threads: usize) -> (Vec<String>, StringAgg) {
    use rayon::prelude::*;

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

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

fn orx_format_and_collect(n: usize, num_threads: usize) -> (Vec<String>, StringAgg) {
    let pairs: Vec<_> = (0..n)
        .into_par()
        .num_threads(num_threads)
        .map(|i| format_number(i as u64))
        .collect();

    let strings = pairs.iter().map(|(s, _)| s.clone()).collect();
    let agg = pairs
        .into_iter()
        .map(|(_, a)| a)
        .reduce(merge_agg)
        .unwrap_or_default();

    (strings, agg)
}

fn orx_fixed_format_and_collect(n: usize, num_threads: usize) -> (Vec<String>, StringAgg) {
    let pairs: Vec<_> = (0..n)
        .into_par()
        .runner(Runner::fixed(Pool::once(num_threads)))
        .num_threads(num_threads)
        .map(|i| format_number(i as u64))
        .collect();

    let strings = pairs.iter().map(|(s, _)| s.clone()).collect();
    let agg = pairs
        .into_iter()
        .map(|(_, a)| a)
        .reduce(merge_agg)
        .unwrap_or_default();

    (strings, agg)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Output {
    string_count: u64,
    agg: StringAgg,
}

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
            Method::Seq => seq_format_and_collect(*input),
            Method::Rayon { nt } => rayon_format_and_collect(*input, *nt),
            Method::Orx { nt } => orx_format_and_collect(*input, *nt),
            Method::OrxFixed { nt } => orx_fixed_format_and_collect(*input, *nt),
        };

        Output {
            string_count: strings.len() as u64,
            agg,
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let (strings, agg) = seq_format_and_collect(*input);
        Some(Output {
            string_count: strings.len() as u64,
            agg,
        })
    }
}

fn run(c: &mut Criterion) {
    let treatments: Vec<_> = [1_000_000, 10_000_000]
        .into_iter()
        .map(|size| InputVariant { size })
        .collect();

    let par_variants = |nt: usize| [Method::Rayon { nt }, Method::Orx { nt }, Method::OrxFixed { nt }];

    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "string_formatting_collection", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
