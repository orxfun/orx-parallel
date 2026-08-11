mod alg;
mod exp;
mod input;

use crate::input::{Dataset, InputVariant};
use crate::{alg::Method, exp::Exp};
use clap::Parser;
use orx_parallel_bench_helper::{BenchArgs, runner};

fn main() {
    let args = BenchArgs::parse();

    let ns = [16, 20];
    let datasets = [Dataset::Map, Dataset::Set];

    let input_variants: Vec<_> = ns
        .into_iter()
        .flat_map(|n| datasets.map(|dataset| InputVariant { n, dataset }))
        .collect();

    runner::run(&args, Exp, &input_variants, &Method::get());
}
