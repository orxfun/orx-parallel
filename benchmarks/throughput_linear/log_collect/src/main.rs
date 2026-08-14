mod alg;
mod exp;
mod input;

use crate::{alg::Method, exp::Exp, input::InputVariant};
use bench_helper::{runner, BenchArgs};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();
    let ns = [16, 20];
    let heavy_options = [false, true];
    let input_variants: Vec<_> = ns.into_iter().flat_map(|n| heavy_options.map(|heavy| InputVariant { n, heavy })).collect();
    runner::run(&args, Exp, &input_variants, &Method::get());
}
