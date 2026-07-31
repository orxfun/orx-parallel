use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::IterationOrder::Arbitrary;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;
use std::hint::black_box;

const FIB_UPPER_BOUND: u64 = 501;

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

struct InputVariant {
    n: usize,
    heavy: bool,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
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

fn run_seq(input: &[u64], heavy: bool) -> Output {
    Output::Vec(match heavy {
        true => input.iter().map(h_m).collect(),
        false => input.iter().map(l_m).collect(),
    })
}

fn run_rayon(input: &[u64], heavy: bool, nt: usize, list: bool) -> Output {
    let pool = ThreadPoolBuilder::new().num_threads(nt).build().unwrap();
    match (heavy, list) {
        (true, false) => Output::Vec(pool.install(|| input.into_par_iter().map(h_m).collect())),
        (false, false) => Output::Vec(pool.install(|| input.into_par_iter().map(l_m).collect())),
        (true, true) => {
            Output::VecList(pool.install(|| input.into_par_iter().map(h_m).collect_vec_list()))
        }
        (false, true) => {
            Output::VecList(pool.install(|| input.into_par_iter().map(l_m).collect_vec_list()))
        }
    }
}

fn run_orx(
    input: &[u64],
    heavy: bool,
    fixed: bool,
    nt: usize,
    ord: IterationOrder,
    list: bool,
) -> Output {
    match heavy {
        true => {
            let par = input
                .into_par()
                .num_threads(nt)
                .iteration_order(ord)
                .map(h_m);
            match (fixed, list) {
                (false, false) => Output::Vec(par.collect()),
                (false, true) => Output::VecVec(par.collect::<Vec2<_>>().into()),
                (true, false) => Output::Vec(par.runner(Runner::fixed(Pool::once(nt))).collect()),
                (true, true) => Output::VecVec(
                    par.runner(Runner::fixed(Pool::once(nt)))
                        .collect::<Vec2<_>>()
                        .into(),
                ),
            }
        }
        false => {
            let par = input
                .into_par()
                .num_threads(nt)
                .iteration_order(ord)
                .map(l_m);
            match (fixed, list) {
                (false, false) => Output::Vec(par.collect()),
                (false, true) => Output::VecVec(par.collect::<Vec2<_>>().into()),
                (true, false) => Output::Vec(par.runner(Runner::fixed(Pool::once(nt))).collect()),
                (true, true) => Output::VecVec(
                    par.runner(Runner::fixed(Pool::once(nt)))
                        .collect::<Vec2<_>>()
                        .into(),
                ),
            }
        }
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
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let heavy = input_variant.heavy;

        match alg_variant {
            Method::SeqVec => (true, run_seq(input, heavy)),
            Method::RayonVec { nt } => (true, run_rayon(input, heavy, *nt, false)),
            Method::RayonVecList { nt } => (false, run_rayon(input, heavy, *nt, true)),
            Method::OrxVec { nt } => (
                true,
                run_orx(input, heavy, false, *nt, IterationOrder::Ordered, false),
            ),
            Method::OrxArbVec { nt } => {
                (false, run_orx(input, heavy, false, *nt, Arbitrary, false))
            }
            Method::OrxArbVecVec { nt } => {
                (false, run_orx(input, heavy, false, *nt, Arbitrary, true))
            }
            Method::OrxVecFixed { nt } => (
                true,
                run_orx(input, heavy, true, *nt, IterationOrder::Ordered, false),
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
    let ns = [15, 20];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| {
            [
                InputVariant { n, heavy: false },
                InputVariant { n, heavy: true },
            ]
        })
        .collect();

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

    Exp.bench(c, "collect_m", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
