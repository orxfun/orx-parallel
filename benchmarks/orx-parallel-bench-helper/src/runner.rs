use crate::BenchArgs;
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
            alg_variant.key_short(),
            input_variant.key_short(),
            elapsed_ns_per_run
        );
    }
}

pub fn list_inputs<E: Experiment>() {
    let keys = <E::InputFactors as Factors>::factor_names_short();
    println!("{keys:?}");
}
