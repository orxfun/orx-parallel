mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::BenchArgs;
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let num_bodies = [1 << 10, 1 << 12];
    let steps = [10, 50];

    let input_variants: Vec<_> = num_bodies
        .into_iter()
        .flat_map(|bodies| {
            steps.into_iter().map(move |steps| InputVariant {
                num_bodies: bodies,
                steps,
            })
        })
        .collect();

    bench_helper::runner::run(&args, Exp, &input_variants, &Method::get());
}
