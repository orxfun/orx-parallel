mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::BenchArgs;
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let num_items = [50, 100];
    let population_sizes = [10_000, 50_000];
    let heavy = [false, true];

    let for_items_pop = |num_items, population_size| {
        heavy.map(|heavy| InputVariant {
            num_items,
            population_size,
            heavy,
        })
    };
    let for_items = |items| {
        population_sizes
            .into_iter()
            .flat_map(move |pop| for_items_pop(items, pop))
    };
    let input_variants: Vec<_> = num_items.into_iter().flat_map(for_items).collect();

    bench_helper::runner::run(&args, Exp, &input_variants, &Method::get());
}
