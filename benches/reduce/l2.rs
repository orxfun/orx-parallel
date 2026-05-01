use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const FIB_UPPER_BOUND: u64 = 301;

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

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

fn l_r(a: u64, b: u64) -> u64 {
    a + b
}

fn h_r(a: u64, b: u64) -> u64 {
    let f = black_box(fibonacci(a % FIB_UPPER_BOUND));
    let g = black_box(a + f);
    g + b - f
}

fn l_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| 2 * x + a)
}

fn h_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| fibonacci((x + a) % FIB_UPPER_BOUND))
}

#[derive(Clone, Copy)]
struct Input {
    n: usize,
    heavy: bool,
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
    Seq,
    Rayon,
    RayonRedWith,
    Orx,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::Seq => "seq",
                Self::Rayon => "rayon",
                Self::RayonRedWith => "rayon-reduce-with",
                Self::Orx => "orx",
            }
            .to_string(),
        ]
    }
}

struct Exp;

impl Experiment for Exp {
    type InputFactors = Input;

    type AlgFactors = Method;

    type Input = (Input, Vec<u64>);

    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        (*input_variant, inputs(1 << input_variant.n))
    }

    fn execute(&mut self, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        let h = input.0.heavy;
        let input = input.1.as_slice();
        match alg_variant {
            Method::Seq => match h {
                true => input.iter().flat_map(l_l).reduce(h_r),
                false => input.iter().flat_map(h_l).reduce(l_r),
            },
            Method::RayonRedWith => match h {
                true => input.into_par_iter().flat_map_iter(l_l).reduce_with(h_r),
                false => input.into_par_iter().flat_map_iter(h_l).reduce_with(l_r),
            },
            Method::Rayon => Some(match h {
                true => input.into_par_iter().flat_map_iter(l_l).reduce(|| 0, h_r),
                false => input.into_par_iter().flat_map_iter(h_l).reduce(|| 0, l_r),
            }),
            Method::Orx => match h {
                true => input.into_par().flat_map(l_l).reduce(h_r),
                false => input.into_par().flat_map(h_l).reduce(l_r),
            },
        }
    }

    fn expected_output(
        &self,
        _: &Self::InputFactors,
        (input_variant, input): &Self::Input,
    ) -> Option<Self::Output> {
        Some(match input_variant.heavy {
            true => input.iter().flat_map(l_l).reduce(h_r),
            false => input.iter().flat_map(h_l).reduce(l_r),
        })
    }
}

fn run(c: &mut Criterion) {
    let treatments = [
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

    Exp.bench(c, "reduce_l", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
