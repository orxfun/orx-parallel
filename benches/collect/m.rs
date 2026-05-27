use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;
use std::hint::black_box;

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

fn l_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn h_m(x: &u64) -> u64 {
    let f = black_box(fibonacci(*x % FIB_UPPER_BOUND));
    let g = black_box(*x + f);
    match *x {
        999 => g - f,
        n => 7 * n + 1000,
    }
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
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let h = input_variant.heavy;

        match alg_variant {
            Method::SeqVec => (
                true,
                Output::Vec(match h {
                    true => input.iter().map(h_m).collect(),
                    false => input.iter().map(l_m).collect(),
                }),
            ),
            Method::RayonVec => (
                true,
                Output::Vec(match h {
                    true => input.into_par_iter().map(h_m).collect(),
                    false => input.into_par_iter().map(l_m).collect(),
                }),
            ),
            Method::RayonVecList => (
                false,
                Output::VecList(match h {
                    true => input.into_par_iter().map(h_m).collect_vec_list(),
                    false => input.into_par_iter().map(l_m).collect_vec_list(),
                }),
            ),
            Method::OrxVecFix => (
                true,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .map(h_m)
                        .collect(),
                    false => input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .map(l_m)
                        .collect(),
                }),
            ),
            Method::OrxArbVecFix => (
                false,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(h_m)
                        .collect(),
                    false => input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(l_m)
                        .collect(),
                }),
            ),
            Method::OrxArbVecVecFix => (
                false,
                Output::VecVec(match h {
                    true => input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(h_m)
                        .collect(),
                    false => input
                        .into_par()
                        .runner(Runner::fixed_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(l_m)
                        .collect(),
                }),
            ),
            Method::OrxVecDyn => (
                true,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .map(h_m)
                        .collect(),
                    false => input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .map(l_m)
                        .collect(),
                }),
            ),
            Method::OrxArbVecDyn => (
                false,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(h_m)
                        .collect(),
                    false => input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(l_m)
                        .collect(),
                }),
            ),
            Method::OrxArbVecVecDyn => (
                false,
                Output::VecVec(match h {
                    true => input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(h_m)
                        .collect(),
                    false => input
                        .into_par()
                        .runner(Runner::dynamic_chunk(Pool::once(0)))
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(l_m)
                        .collect(),
                }),
            ),
        }
    }

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
        (ordered, output): &Self::Output,
    ) {
        let mut expected: Vec<_> = match input_variant.heavy {
            true => input.iter().map(h_m).collect(),
            false => input.iter().map(l_m).collect(),
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

    Exp.bench(c, "col_m", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
