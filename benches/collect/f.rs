use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{collections::LinkedList, hint::black_box};

fn f(a: &u64) -> bool {
    !black_box((a + 7).is_multiple_of(11))
}

struct InputVariant {
    n: usize,
    num_threads: usize,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("2e{}", self.n), self.num_threads.to_string()]
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
                Self::OrxArbVecVec => "orx-arb-vec2",
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
        match alg_variant {
            Method::SeqVec => (true, Output::Vec(input.iter().copied().filter(f).collect())),
            Method::RayonVec => {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    (
                        true,
                        Output::Vec(input.into_par_iter().copied().filter(f).collect()),
                    )
                })
            }
            Method::RayonVecList => {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    (
                        false,
                        Output::VecList(input.into_par_iter().copied().filter(f).collect_vec_list()),
                    )
                })
            }
            Method::OrxVec => (
                true,
                Output::Vec(
                    input
                        .into_par()
                        .num_threads(input_variant.num_threads)
                        .copied()
                        .filter(f)
                        .collect(),
                ),
            ),
            Method::OrxArbVec => (
                false,
                Output::Vec(
                    input
                        .into_par()
                        .num_threads(input_variant.num_threads)
                        .iteration_order(IterationOrder::Arbitrary)
                        .copied()
                        .filter(f)
                        .collect(),
                ),
            ),
            Method::OrxArbVecVec => (
                false,
                Output::VecVec(
                    input
                        .into_par()
                        .num_threads(input_variant.num_threads)
                        .iteration_order(IterationOrder::Arbitrary)
                        .copied()
                        .filter(f)
                        .collect(),
                ),
            ),
        }
    }

    fn validate_output(
        &self,
        _: &Self::InputFactors,
        input: &Self::Input,
        (ordered, output): &Self::Output,
    ) {
        let mut expected: Vec<_> = input.iter().copied().filter(f).collect();
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
    let num_threads_options = [16, 32];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| [InputVariant { n: 15, num_threads }, InputVariant { n: 20, num_threads }])
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "col_f", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
