mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();
    let ns = [16, 20];
    let trace_modes = [false, true];
    let input_variants: Vec<_> = ns
        .into_iter()
        .flat_map(|n| trace_modes.map(|with_trace| InputVariant { n, with_trace }))
        .collect();
    runner::run(&args, Exp, &input_variants, &Method::get());
}
