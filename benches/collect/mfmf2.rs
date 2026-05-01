/*
# col_mfmf
+----+---+---+------+-------+----------------+-----------+
| t  | i | a | n    | task  | method         | Time (ns) |
+----+---+---+------+-------+----------------+-----------+
| 1  | 1 | 1 | 2e15 | light | seq-vec        |    102414 |
+----+---+---+------+-------+----------------+-----------+
| 2  | 1 | 2 | 2e15 | light | rayon-vec      |  12667150 |
+----+---+---+------+-------+----------------+-----------+
| 3  | 1 | 3 | 2e15 | light | rayon-veclist  |  11825165 |
+----+---+---+------+-------+----------------+-----------+
| 4  | 1 | 4 | 2e15 | light | orx-vec        |   1570339 |
+----+---+---+------+-------+----------------+-----------+
| 5  | 1 | 5 | 2e15 | light | orx-arb-vec    |   1545366 |
+----+---+---+------+-------+----------------+-----------+
| 6  | 1 | 6 | 2e15 | light | orx-arb-vecvec |   1528301 |
+----+---+---+------+-------+----------------+-----------+
| 7  | 2 | 1 | 2e20 | light | seq-vec        |   3094266 |
+----+---+---+------+-------+----------------+-----------+
| 8  | 2 | 2 | 2e20 | light | rayon-vec      |  19132906 |
+----+---+---+------+-------+----------------+-----------+
| 9  | 2 | 3 | 2e20 | light | rayon-veclist  |  17543174 |
+----+---+---+------+-------+----------------+-----------+
| 10 | 2 | 4 | 2e20 | light | orx-vec        |   3635643 |
+----+---+---+------+-------+----------------+-----------+
| 11 | 2 | 5 | 2e20 | light | orx-arb-vec    |   3945630 |
+----+---+---+------+-------+----------------+-----------+
| 12 | 2 | 6 | 2e20 | light | orx-arb-vecvec |   2985271 |
+----+---+---+------+-------+----------------+-----------+
| 13 | 3 | 1 | 2e15 | heavy | seq-vec        |   1329243 |
+----+---+---+------+-------+----------------+-----------+
| 14 | 3 | 2 | 2e15 | heavy | rayon-vec      |  11799816 |
+----+---+---+------+-------+----------------+-----------+
| 15 | 3 | 3 | 2e15 | heavy | rayon-veclist  |  11786519 |
+----+---+---+------+-------+----------------+-----------+
| 16 | 3 | 4 | 2e15 | heavy | orx-vec        |   2091251 |
+----+---+---+------+-------+----------------+-----------+
| 17 | 3 | 5 | 2e15 | heavy | orx-arb-vec    |   2139048 |
+----+---+---+------+-------+----------------+-----------+
| 18 | 3 | 6 | 2e15 | heavy | orx-arb-vecvec |   2090661 |
+----+---+---+------+-------+----------------+-----------+
| 19 | 4 | 1 | 2e20 | heavy | seq-vec        |  39306389 |
+----+---+---+------+-------+----------------+-----------+
| 20 | 4 | 2 | 2e20 | heavy | rayon-vec      |  21391458 |
+----+---+---+------+-------+----------------+-----------+
| 21 | 4 | 3 | 2e20 | heavy | rayon-veclist  |  19719572 |
+----+---+---+------+-------+----------------+-----------+
| 22 | 4 | 4 | 2e20 | heavy | orx-vec        |   6683224 |
+----+---+---+------+-------+----------------+-----------+
| 23 | 4 | 5 | 2e20 | heavy | orx-arb-vec    |   6730883 |
+----+---+---+------+-------+----------------+-----------+
| 24 | 4 | 6 | 2e20 | heavy | orx-arb-vecvec |   6830089 |
+----+---+---+------+-------+----------------+-----------+
*/

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

fn m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn f(a: &u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn h_m2(x: u64) -> u64 {
    match x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    }
}

fn l_m2(x: u64) -> u64 {
    match x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn f2(a: &u64) -> bool {
    !(2 * a + 11).is_multiple_of(7)
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
                    true => input.iter().map(m).filter(f).map(h_m2).filter(f2).collect(),
                    false => input.iter().map(m).filter(f).map(l_m2).filter(f2).collect(),
                }),
            ),
            Method::RayonVec => (
                true,
                Output::Vec(match h {
                    true => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .collect(),
                    false => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .collect(),
                }),
            ),
            Method::RayonVecList => (
                false,
                Output::VecList(match h {
                    true => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .collect_vec_list(),
                    false => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .collect_vec_list(),
                }),
            ),
            Method::OrxVec => (
                true,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .collect(),
                    false => input
                        .into_par()
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .collect(),
                }),
            ),
            Method::OrxArbVec => (
                false,
                Output::Vec(match h {
                    true => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .collect(),
                    false => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .collect(),
                }),
            ),
            Method::OrxArbVecVec => (
                false,
                Output::VecVec(match h {
                    true => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .collect(),
                    false => input
                        .into_par()
                        .iteration_order(IterationOrder::Arbitrary)
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
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
            true => input.iter().map(m).filter(f).map(h_m2).filter(f2).collect(),
            false => input.iter().map(m).filter(f).map(l_m2).filter(f2).collect(),
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

    Exp.bench(c, "col_mfmf", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
