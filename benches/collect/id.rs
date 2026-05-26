use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;

struct Input {
    n: usize,
}

impl Input {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("2e{}", self.n)]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    SeqVec,
    RayonVec,
    RayonVecList,
    OrxVecFix,
    OrxArbVecFix,
    OrxArbVecVecFix,
    OrxVecDyn,
    OrxArbVecDyn,
    OrxArbVecVecDyn,
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
                Self::OrxVecFix => "orx-vec-fix",
                Self::OrxArbVecFix => "orx-arb-vec-fix",
                Self::OrxArbVecVecFix => "orx-arb-vec2-fix",
                Self::OrxVecDyn => "orx-vec-dyn",
                Self::OrxArbVecDyn => "orx-arb-vec-dyn",
                Self::OrxArbVecVecDyn => "orx-arb-vec2-dyn",
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
            Method::SeqVec => (true, Output::Vec(input.iter().copied().collect())),
            Method::RayonVec => (true, Output::Vec(input.into_par_iter().copied().collect())),
            Method::RayonVecList => (
                false,
                Output::VecList(input.into_par_iter().copied().collect_vec_list()),
            ),
            Method::OrxVecFix => (
                true,
                Output::Vec(
                    input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .copied()
                        .collect(),
                ),
            ),
            Method::OrxArbVecFix => (
                false,
                Output::Vec(
                    input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .copied()
                        .collect(),
                ),
            ),
            Method::OrxArbVecVecFix => (
                false,
                Output::VecVec(
                    input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .copied()
                        .collect(),
                ),
            ),
            Method::OrxVecDyn => (
                true,
                Output::Vec(
                    input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .copied()
                        .collect(),
                ),
            ),
            Method::OrxArbVecDyn => (
                false,
                Output::Vec(
                    input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .copied()
                        .collect(),
                ),
            ),
            Method::OrxArbVecVecDyn => (
                false,
                Output::VecVec(
                    input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .copied()
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
        let mut expected: Vec<_> = input.iter().copied().collect();
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
    let treatments = vec![Input { n: 15 }, Input { n: 20 }];

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "col_id", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
