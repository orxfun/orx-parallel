mod alg;
mod exp;
mod input;

use crate::{
    alg::Method,
    exp::Exp,
    input::{InputVariant, InvalidProfile},
};
use bench_helper::{BenchArgs, runner};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let ns = [16, 20];
    let profiles = [
        InvalidProfile::SuccessHeavy,
        InvalidProfile::Mixed,
        InvalidProfile::FailEarly,
    ];

    let input_variants: Vec<_> = ns
        .into_iter()
        .flat_map(|n| profiles.map(|profile| InputVariant { n, profile }))
        .collect();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
