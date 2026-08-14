mod alg;
mod exp;
mod input;

use crate::{
    alg::Method,
    exp::Exp,
    input::{Dist, InputVariant},
};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let ns = [16, 20];
    let distributions = [Dist::Uniform, Dist::Skewed];

    let input_variants: Vec<_> = ns
        .into_iter()
        .flat_map(|n| distributions.map(|dist| InputVariant { n, dist }))
        .collect();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
