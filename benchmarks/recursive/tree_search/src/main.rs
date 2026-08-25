mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();
    let input_variants = [
        InputVariant {
            depth: 6,
            fan_out: 8,
            threshold: 5_000,
        },
        InputVariant {
            depth: 8,
            fan_out: 6,
            threshold: 20_000,
        },
        InputVariant {
            depth: 8,
            fan_out: 8,
            threshold: 10_000,
        },
    ];
    runner::run(&args, Exp, &input_variants, &Method::get());
}
