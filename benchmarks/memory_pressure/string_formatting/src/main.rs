mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use orx_criterion::*;
use std::time::Instant;

fn main() {
    let num_warmup = 3;
    let num_actual = 10;

    let alg_variants = [Method::get()];
    let input_variants = [
        InputVariant { size: 10_000 },
        InputVariant { size: 100_000 },
    ];

    let mut exp = Exp;

    for input_variant in input_variants {
        let input = exp.input(&input_variant);
        for alg_variant in &alg_variants {
            let output = exp.execute(&input_variant, alg_variant, &input);
            if let Some(expected) = exp.expected_output(&input_variant, &input) {
                assert_eq!(output, expected);
            }
            exp.validate_output(&input_variant, &input, &output);

            for _ in 0..num_warmup {
                exp.execute(&input_variant, alg_variant, &input);
            }

            let start = Instant::now();
            for _ in 0..num_actual {
                exp.execute(&input_variant, alg_variant, &input);
            }
            println!(
                "{}\t{}:\t\t{:?}",
                input_variant.key_short(),
                alg_variant.key_short(),
                start.elapsed()
            );
        }
    }
}

// RAYON_NUM_THREADS=4 cargo run

/*
clear; cargo run --release --features seq
clear; RAYON_NUM_THREADS=4 cargo run --release --features rayon
clear; ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features orx-once
clear; ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features orx-basic
clear; ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features orx-rayon
*/
