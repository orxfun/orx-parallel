mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let ns = [10, 12, 14];
    let heterogeneity_levels = [0.001, 0.011, 0.101, 0.201];
    let input_variants = ns
        .into_iter()
        .flat_map(|n| {
            heterogeneity_levels
                .into_iter()
                .map(move |heterogeneity_level| InputVariant {
                    n,
                    heterogeneity_level,
                })
        })
        .collect::<Vec<_>>();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
