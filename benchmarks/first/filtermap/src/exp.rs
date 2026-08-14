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

fn find(x: &u64) -> bool {
    *x > (1 << 25)
}

fn l_f(a: &u64) -> Option<u64> {
    match !black_box((a + 7).is_multiple_of(11)) {
        true => Some(a + 7),
        false => None,
    }
}

fn h_f(a: &u64) -> Option<u64> {
    let fib_val = runner::fib(FIB_UPPER_BOUND, *a);
    match !black_box((fib_val + 7).is_multiple_of(11)) {
        true => Some(fib_val),
        false => None,
    }
}

fn run_seq(input: &[u64], heavy: bool) -> u64 {
    match heavy {
        true => input.iter().filter_map(h_f).find(find),
        false => input.iter().filter_map(l_f).find(find),
    }
    .unwrap()
}

fn run_rayon(input: &[u64], heavy: bool) -> u64 {
    use rayon::prelude::*;
    match heavy {
        true => input.into_par_iter().filter_map(h_f).find_first(find),
        false => input.into_par_iter().filter_map(l_f).find_first(find),
    }
    .unwrap()
}

fn run_orx(input: &[u64], heavy: bool, ord: IterationOrder) -> u64 {
    use orx_parallel::*;
    let par = input.into_par().iteration_order(ord);
    match heavy {
        true => par.filter_map(h_f).find(find),
        false => par.filter_map(l_f).find(find),
    }
    .unwrap()
}
