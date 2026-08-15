mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::BenchArgs;
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let num_items = [50, 100];
    let num_items = [20];
    let population_sizes = [100, 200];

    let combine_populations = |items| {
        population_sizes.map(|pop| InputVariant {
            num_items: items,
            population_size: pop,
        })
    };

    let input_variants: Vec<_> = num_items
        .into_iter()
        .flat_map(combine_populations)
        .collect();

    bench_helper::runner::run(&args, Exp, &input_variants, &Method::get());
}
