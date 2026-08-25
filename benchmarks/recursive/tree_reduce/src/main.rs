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
            depth: 7,
            fanout: 3,
        },
        InputVariant {
            depth: 8,
            fanout: 3,
        },
        InputVariant {
            depth: 6,
            fanout: 5,
        },
    ];
    runner::run(&args, Exp, &input_variants, &Method::get());
}
