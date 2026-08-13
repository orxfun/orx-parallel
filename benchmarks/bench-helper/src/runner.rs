use crate::{BenchArgs, RunMode};
use orx_criterion::{Experiment, Factors};
use std::{hint::black_box, time::Instant};

pub fn run_benchmark<E>(
    args: &BenchArgs,
    mut exp: E,
    input_variants: &[E::InputFactors],
    alg_variant: &E::AlgFactors,
) where
    E: Experiment,
{
    for input_variant in input_variants {
        let input = exp.input(&input_variant);
        // validation
        let output = exp.execute(&input_variant, alg_variant, &input);
        if let Some(expected) = exp.expected_output(&input_variant, &input) {
            assert_eq!(output, expected);
        }
        exp.validate_output(&input_variant, &input, &output);

        // warmup
        for _ in 0..args.warmup_runs {
            black_box(exp.execute(&input_variant, alg_variant, &input));
        }

        // actual
        let start = Instant::now();
        for _ in 0..args.actual_runs {
            black_box(exp.execute(&input_variant, alg_variant, &input));
        }
        let elapsed_ns = start.elapsed().as_nanos();
        let elapsed_ns_per_run = elapsed_ns / args.actual_runs as u128;

        println!(
            "{}__{}__{}",
            alg_variant.key_short().replace("_", "__"),
            input_variant.key_short().replace("_", "__"),
            elapsed_ns_per_run
        );
    }
}

pub fn list_inputs<E: Experiment>() {
    let keys = <E::InputFactors as Factors>::factor_names_short();
    println!("{keys:?}");
}

pub fn list_methods<E: Experiment>() {
    let keys = <E::AlgFactors as Factors>::factor_names_short();
    println!("{keys:?}");
}

pub fn run<E: Experiment>(
    args: &BenchArgs,
    exp: E,
    input_variants: &[E::InputFactors],
    alg_variant: &E::AlgFactors,
) {
    match args.run_mode {
        RunMode::ListInputs => list_inputs::<E>(),
        RunMode::ListMethods => list_methods::<E>(),
        RunMode::Run => run_benchmark(&args, exp, &input_variants, &alg_variant),
    }
}

pub fn cpu_mix(rounds: usize, seed: u64) -> u64 {
    let mut x = black_box(seed ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..rounds {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

pub fn fib(upper_bound: u64, n: u64) -> u64 {
    let n = black_box(n % upper_bound);
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = black_box(a + b);
        a = black_box(b);
        b = black_box(c);
    }
    a
}
