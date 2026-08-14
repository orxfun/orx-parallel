mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::{InputVariant, Pos}};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let ns = [14, 18];
    let positions = [Pos::Never, Pos::Late, Pos::Mid, Pos::Early];

    let input_variants: Vec<_> = ns
        .into_iter()
        .flat_map(|n| positions.map(|pos| InputVariant { n, pos }))
        .collect();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
