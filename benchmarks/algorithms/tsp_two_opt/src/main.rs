mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::BenchArgs;
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let num_cities = [20, 50];
    let iterations = [100, 500];

    let combine_iterations = |cities| {
        iterations.map(|iters| InputVariant {
            num_cities: cities,
            iterations: iters,
        })
    };

    let input_variants: Vec<_> = num_cities
        .into_iter()
        .flat_map(combine_iterations)
        .collect();

    bench_helper::runner::run(&args, Exp, &input_variants, &Method::get());
}
