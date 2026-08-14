mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let input_variants = [
        InputVariant { size: 10_000 },
        InputVariant { size: 100_000 },
    ];

    runner::run(&args, Exp, &input_variants, &Method::get());
}
