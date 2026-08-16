mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::BenchArgs;
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let num_items = [50, 100];
    let restarts = [100, 500];
    let steps = 1_000;

    let input_variants: Vec<_> = num_items
        .into_iter()
        .flat_map(|items| {
            restarts.into_iter().map(move |restarts| InputVariant {
                num_items: items,
                restarts,
                steps,
            })
        })
        .collect();

    bench_helper::runner::run(&args, Exp, &input_variants, &Method::get());
}
