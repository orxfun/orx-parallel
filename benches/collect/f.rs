use criterion::{Criterion, criterion_group, criterion_main};
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
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("2e{}", self.n)]
    }
}

enum Method {
    SeqVec,
    RayonVec { nt: usize },
    RayonVecList { nt: usize },
    OrxVec { nt: usize },
    OrxArbVec { nt: usize },
    OrxArbVecVec { nt: usize },
    OrxVecFixed { nt: usize },
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::SeqVec => "seq-vec".to_string(),
            Self::RayonVec { nt } => format!("rayon-vec-{nt}"),
            Self::RayonVecList { nt } => format!("rayon-veclist-{nt}"),
            Self::OrxVec { nt } => format!("orx-vec-{nt}"),
            Self::OrxArbVec { nt } => format!("orx-arb-vec-{nt}"),
            Self::OrxArbVecVec { nt } => format!("orx-arb-vec2-{nt}"),
            Self::OrxVecFixed { nt } => format!("orx-vec-fixed-{nt}"),
        }]
    }
}

#[derive(Debug, PartialEq)]
enum Output {
    Vec(Vec<u64>),
    VecList(LinkedList<Vec<u64>>),
    VecVec(Vec<Vec<u64>>),
}

fn run_seq(input: &[u64]) -> Output {
    Output::Vec(input.iter().copied().filter(f).collect())
}

fn run_rayon(input: &[u64], nt: usize, list: bool) -> Output {
    let pool = ThreadPoolBuilder::new().num_threads(nt).build().unwrap();
    match list {
        false => Output::Vec(pool.install(|| input.into_par_iter().copied().filter(f).collect())),
        true => Output::VecList(
            pool.install(|| input.into_par_iter().copied().filter(f).collect_vec_list()),
        ),
    }
}

fn run_orx(input: &[u64], fixed: bool, nt: usize, ord: IterationOrder, list: bool) -> Output {
    let par = input
        .into_par()
        .num_threads(nt)
        .iteration_order(ord)
        .copied()
        .filter(f);
    match (fixed, list) {
        (false, false) => Output::Vec(par.collect()),
        (false, true) => Output::VecVec(par.collect::<Vec2<_>>().into()),
        (true, false) => Output::Vec(par.runner(Runner::fixed()).collect()),
        (true, true) => Output::VecVec(
            par.runner(Runner::fixed())
                .collect::<Vec2<_>>()
                .into(),
        ),
    }
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
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::SeqVec => (true, run_seq(input)),
            Method::RayonVec { nt } => (true, run_rayon(input, *nt, false)),
            Method::RayonVecList { nt } => (false, run_rayon(input, *nt, true)),
            Method::OrxVec { nt } => (
                true,
                run_orx(input, false, *nt, IterationOrder::Ordered, false),
            ),
            Method::OrxArbVec { nt } => (
                false,
                run_orx(input, false, *nt, IterationOrder::Arbitrary, false),
            ),
            Method::OrxArbVecVec { nt } => (
                false,
                run_orx(input, false, *nt, IterationOrder::Arbitrary, true),
            ),
            Method::OrxVecFixed { nt } => (
                true,
                run_orx(input, true, *nt, IterationOrder::Ordered, false),
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
    let ns = [16, 20];
    let treatments: Vec<_> = ns.into_iter().map(|n| InputVariant { n }).collect();

    let par_variants = |nt: usize| {
        [
            Method::RayonVec { nt },
            Method::RayonVecList { nt },
            Method::OrxVec { nt },
            Method::OrxArbVec { nt },
            Method::OrxArbVecVec { nt },
            Method::OrxVecFixed { nt },
        ]
    };
    let mut variants = vec![Method::SeqVec];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "collect_f", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
