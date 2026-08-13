use crate::{alg::Method, input::InputVariant};
use bench_helper::runner;
use orx_criterion::Experiment;
use orx_parallel::IterationOrder;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{collections::LinkedList, hint::black_box};

const FIB_UPPER_BOUND: u64 = 99;

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
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => (true, run_seq(input)),
            Method::Rayon => (true, run_rayon(input, false)),
            Method::RayonVec2 => (false, run_rayon(input, true)),
            Method::OrxOnce => (true, run_orx(input, IterationOrder::Ordered, false)),
            Method::OrxBasic => (true, run_orx(input, IterationOrder::Ordered, false)),
            Method::OrxRayon => (true, run_orx(input, IterationOrder::Ordered, false)),
            Method::OrxOnceVec2 => (false, run_orx(input, IterationOrder::Ordered, true)),
            Method::OrxBasicVec2 => (false, run_orx(input, IterationOrder::Ordered, true)),
            Method::OrxRayonVec2 => (false, run_orx(input, IterationOrder::Ordered, true)),
        }
    }

    fn validate_output(
        &self,
        _: &Self::InputFactors,
        input: &Self::Input,
        (ordered, output): &Self::Output,
    ) {
        let mut expected = match run_seq(input) {
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

fn f(a: &u64) -> bool {
    let a = runner::fib(FIB_UPPER_BOUND, *a);
    !black_box((a + 7).is_multiple_of(11))
}

fn run_seq(input: &[u64]) -> Output {
    Output::Vec(input.iter().copied().filter(f).collect())
}

fn run_rayon(input: &[u64], list: bool) -> Output {
    use rayon::prelude::*;
    match list {
        false => Output::Vec(input.into_par_iter().copied().filter(f).collect()),
        true => Output::VecList(input.into_par_iter().copied().filter(f).collect_vec_list()),
    }
}

fn run_orx(input: &[u64], ord: IterationOrder, list: bool) -> Output {
    use orx_parallel::*;
    let par = input.into_par().iteration_order(ord).copied().filter(f);
    match list {
        false => Output::Vec(par.collect()),
        true => Output::VecVec(par.collect::<Vec2<_>>().into()),
    }
}
