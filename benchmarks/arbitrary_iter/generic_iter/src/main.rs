mod alg;
mod exp;
mod input;

use crate::input::InputVariant;
use crate::{alg::Method, exp::Exp};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let ns = [16, 20];
    let input_variants: Vec<_> = ns.into_iter().map(|n| InputVariant { n }).collect();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
