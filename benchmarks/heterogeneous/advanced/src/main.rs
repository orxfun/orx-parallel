mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let sizes = [1 << 12, 1 << 18];
    let heavy_options = [false, true];
    let heterogeneity_options = [2, 5, 10];
    let input_variants = sizes
        .into_iter()
        .flat_map(|n| {
            heavy_options.into_iter().flat_map(move |heavy| {
                heterogeneity_options
                    .into_iter()
                    .map(move |heterogeneity_percent| InputVariant {
                        n,
                        heavy,
                        heterogeneity_percent,
                    })
            })
        })
        .collect::<Vec<_>>();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
