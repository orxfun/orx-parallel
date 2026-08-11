mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use clap::Parser;
use orx_parallel_bench_helper::{BenchArgs, runner};

fn main() {
    let args = BenchArgs::parse();

    let input_variants = [
        InputVariant { size: 10_000 },
        InputVariant { size: 100_000 },
    ];

    match args.list_inputs {
        true => runner::list_inputs::<Exp>(),
        false => runner::run_benchmark(&args, Exp, &input_variants, &Method::get()),
    }
}

// RAYON_NUM_THREADS=4 cargo run

/*
clear; cargo run --release --features seq -- --warmup-runs 4 --actual-runs 20
clear; RAYON_NUM_THREADS=4 ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features rayon -- --warmup-runs 4 --actual-runs 20
clear; RAYON_NUM_THREADS=4 ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features orx-once -- --warmup-runs 4 --actual-runs 20
clear; RAYON_NUM_THREADS=4 ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features orx-basic -- --warmup-runs 4 --actual-runs 20
clear; RAYON_NUM_THREADS=4 ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --features orx-rayon -- --warmup-runs 4 --actual-runs 20
*/
