use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;

const FIB_UPPER_BOUND: u64 = 301;

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn h_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| fibonacci((x + a) % FIB_UPPER_BOUND))
}

fn l_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| 2 * x + a)
}

struct Input {
    n: usize,
    heavy: bool,
}

impl Input {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "task"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.heavy {
                true => "heavy",
                false => "light",
            }
            .to_string(),
        ]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    SeqVec,
    RayonVec,
    RayonVecList,
    OrxVec,
    OrxArbVec,
    OrxArbVecVec,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::SeqVec => "seq-vec",
                Self::RayonVec => "rayon-vec",
                Self::RayonVecList => "rayon-veclist",
                Self::OrxVec => "orx-vec",
                Self::OrxArbVec => "orx-arb-vec",
                Self::OrxArbVecVec => "orx-arb-vecvec",
            }
            .to_string(),
        ]
    }
}

#[derive(Debug, PartialEq)]
enum Output {
    Vec(Vec<u64>),
    VecList(LinkedList<Vec<u64>>),
    VecVec(Vec<Vec<u64>>),
}

struct Exp;

impl Experiment for Exp {
    type InputFactors = Input;

    type AlgFactors = Method;

    type Input = (bool, Vec<u64>); // (heavy, input)

    type Output = (bool, Output); // (ordered, output)

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        const SEED: u64 = 654;
        let len = input_variant.len();
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        (
            input_variant.heavy,
            (0..len).map(|_| rng.random_range(0..150)).collect(),
        )
    }

    fn execute(&mut self, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        let (h, input) = input;

        match alg_variant {
            Method::SeqVec => (
                true,
                Output::Vec(match h {
                    true => input.iter().flat_map(h_l).collect(),
                    false => input.iter().flat_map(l_l).collect(),
                }),
            ),
            Method::RayonVec => (
                true,
                Output::Vec(match h {
                    true => input.into_par_iter().flat_map_iter(h_l).collect(),
                    false => input.into_par_iter().flat_map_iter(l_l).collect(),
                }),
            ),
            Method::RayonVecList => (
                false,
                Output::VecList(match h {
                    true => input.into_par_iter().flat_map_iter(h_l).collect_vec_list(),
                    false => input.into_par_iter().flat_map_iter(l_l).collect_vec_list(),
                }),
            ),
            Method::OrxVec => (
                true,
                Output::Vec(match h {
                    true => input.into_par().flat_map(h_l).collect(),
                    false => input.into_par().flat_map(l_l).collect(),
                }),
            ),
            Method::OrxArbVec => (
                false,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .flat_map(h_l)
                        .collect(),
                    false => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .flat_map(l_l)
                        .collect(),
                }),
            ),
            Method::OrxArbVecVec => (
                false,
                Output::VecVec(match h {
                    true => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .flat_map(h_l)
                        .collect(),
                    false => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .flat_map(l_l)
                        .collect(),
                }),
            ),
        }
    }

    fn validate_output(
        &self,
        _: &Self::InputFactors,
        (h, input): &Self::Input,
        (ordered, output): &Self::Output,
    ) {
        let mut expected: Vec<_> = match h {
            true => input.iter().flat_map(h_l).collect(),
            false => input.iter().flat_map(l_l).collect(),
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

fn run(c: &mut Criterion) {
    let treatments = vec![
        Input {
            n: 15,
            heavy: false,
        },
        Input {
            n: 20,
            heavy: false,
        },
        Input { n: 15, heavy: true },
        Input { n: 20, heavy: true },
    ];

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "col_l", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
