use crate::{alg::Method, input::InputVariant};
use bench_helper::runner;
use orx_criterion::Experiment;
use orx_parallel::IterationOrder;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

const FIB_UPPER_BOUND: u64 = 99;

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<u64>;

    type Output = u64;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        const SEED: u64 = 654;
        let len = input_variant.len();
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        (0..len).map(|_| rng.random_range(0..150)).collect()
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let h = input_variant.heavy;
        match alg_variant {
            Method::Seq => run_seq(input, h),
            Method::Rayon => run_rayon(input, h),
            Method::OrxOnce => run_orx(input, h, IterationOrder::Ordered),
            Method::OrxBasic => run_orx(input, h, IterationOrder::Ordered),
            Method::OrxRayon => run_orx(input, h, IterationOrder::Ordered),
        }
    }

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let expected = run_seq(input, input_variant.heavy);
        assert_eq!(expected, *output);
    }
}

fn l_r(a: u64, b: u64) -> u64 {
    a + b
}

fn h_r(a: u64, b: u64) -> u64 {
    let f = black_box(runner::fib(FIB_UPPER_BOUND, a));
    let g = black_box(a + f);
    g + b - f
}

fn l_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn h_m(x: &u64) -> u64 {
    let f = black_box(runner::fib(FIB_UPPER_BOUND, *x));
    let g = black_box(*x + f);
    match *x {
        999 => g - f,
        n => 7 * n + 1000,
    }
}

fn run_seq(input: &[u64], heavy: bool) -> u64 {
    match heavy {
        true => input.iter().map(h_m).reduce(h_r),
        false => input.iter().map(l_m).reduce(l_r),
    }
    .unwrap()
}

fn run_rayon(input: &[u64], heavy: bool) -> u64 {
    use rayon::prelude::*;
    match heavy {
        true => input.into_par_iter().map(h_m).reduce_with(h_r),
        false => input.into_par_iter().map(l_m).reduce_with(l_r),
    }
    .unwrap()
}

fn run_orx(input: &[u64], heavy: bool, ord: IterationOrder) -> u64 {
    use orx_parallel::*;
    let par = input.into_par().iteration_order(ord);
    match heavy {
        true => par.map(h_m).reduce(h_r),
        false => par.map(l_m).reduce(l_r),
    }
    .unwrap()
}
