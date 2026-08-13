use crate::{alg::Method, input::InputVariant};
use bench_helper::runner;
use orx_criterion::Experiment;
use orx_parallel::IterationOrder;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{collections::LinkedList, hint::black_box};

const FIB_UPPER_BOUND: u64 = 57;

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<u64>;

    type Output = (bool, Output); // (ordered, output)

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
            Method::Seq => (true, run_seq(input, h)),
            Method::Rayon => (true, run_rayon(input, h, false)),
            Method::RayonVec2 => (false, run_rayon(input, h, true)),
            Method::OrxOnce => (true, run_orx(input, h, IterationOrder::Ordered, false)),
            Method::OrxBasic => (true, run_orx(input, h, IterationOrder::Ordered, false)),
            Method::OrxRayon => (true, run_orx(input, h, IterationOrder::Ordered, false)),
            Method::OrxOnceVec2 => (false, run_orx(input, h, IterationOrder::Ordered, true)),
            Method::OrxBasicVec2 => (false, run_orx(input, h, IterationOrder::Ordered, true)),
            Method::OrxRayonVec2 => (false, run_orx(input, h, IterationOrder::Ordered, true)),
        }
    }

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
        (ordered, output): &Self::Output,
    ) {
        let mut expected = match run_seq(input, input_variant.heavy) {
            Output::Vec(vec) => vec,
            _ => unreachable!(),
        };

        if !*ordered {
            expected.sort();
        }

        match output {
            Output::Vec(result) => match *ordered {
                false => {
                    let mut result = result.clone();
                    result.sort();
                    assert_eq!(expected, result)
                }
                true => assert_eq!(&expected, result),
            },
            Output::VecList(result) => {
                assert!(!*ordered);
                let mut result: Vec<u64> = result.iter().flat_map(|x| x.iter()).copied().collect();
                result.sort();
                assert_eq!(expected, result);
            }
            Output::VecVec(result) => {
                assert!(!*ordered);
                let mut result: Vec<u64> = result.iter().flat_map(|x| x.iter()).copied().collect();
                result.sort();
                assert_eq!(expected, result);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Output {
    Vec(Vec<u64>),
    VecList(LinkedList<Vec<u64>>),
    VecVec(Vec<Vec<u64>>),
}

fn l_m(x: &u64) -> Option<u64> {
    match *x {
        999 => None,
        n => Some(7 * n + 1000),
    }
}

fn h_m(x: &u64) -> Option<u64> {
    let f = black_box(runner::fib(FIB_UPPER_BOUND, *x));
    let g = black_box(*x + f);
    match (f + g).is_power_of_two() {
        true => None,
        false => Some(7 * g + 1000 - f),
    }
}

fn run_seq(input: &[u64], heavy: bool) -> Output {
    match heavy {
        true => Output::Vec(input.iter().filter_map(h_m).collect()),
        false => Output::Vec(input.iter().filter_map(l_m).collect()),
    }
}

fn run_rayon(input: &[u64], heavy: bool, list: bool) -> Output {
    use rayon::prelude::*;
    match (heavy, list) {
        (true, false) => Output::Vec(input.into_par_iter().filter_map(h_m).collect()),
        (true, true) => Output::VecList(input.into_par_iter().filter_map(h_m).collect_vec_list()),
        (false, false) => Output::Vec(input.into_par_iter().filter_map(l_m).collect()),
        (false, true) => Output::VecList(input.into_par_iter().filter_map(l_m).collect_vec_list()),
    }
}

fn run_orx(input: &[u64], heavy: bool, ord: IterationOrder, list: bool) -> Output {
    use orx_parallel::*;
    let par = input.into_par().iteration_order(ord);
    match (heavy, list) {
        (true, false) => Output::Vec(par.filter_map(h_m).collect()),
        (true, true) => Output::VecVec(par.filter_map(h_m).collect::<Vec2<_>>().into()),
        (false, false) => Output::Vec(par.filter_map(l_m).collect()),
        (false, true) => Output::VecVec(par.filter_map(l_m).collect::<Vec2<_>>().into()),
    }
}
