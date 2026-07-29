use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

fn l_r(a: u64, b: u64) -> u64 {
    a + b
}

fn h_r(a: u64, b: u64) -> u64 {
    let f = black_box(fibonacci(a % FIB_UPPER_BOUND));
    let g = black_box(a + f);
    g + b - f
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

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    heavy: bool,
    num_threads: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "task", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.heavy {
                true => "heavy",
                false => "light",
            }
            .to_string(),
            self.num_threads.to_string(),
        ]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
    RayonRedWith,
    Orx,
    OrxFixed,
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
                Self::RayonRedWith => "rayon-red-with",
                Self::Orx => "orx",
                Self::OrxFixed => "orx-fixed",
            }
            .to_string(),
        ]
    }
}

struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<u64>;

    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let len = 1 << input_variant.n;
        const SEED: u64 = 654;
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
            Method::Seq => self.expected_output(input_variant, input).unwrap(),
            Method::Rayon => {
                let input = input.as_slice();
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    Some(match h {
                        true => input
                            .into_par_iter()
                            .map(m)
                            .filter(f)
                            .map(h_m2)
                            .filter(f2)
                            .reduce(|| 0, h_r),
                        false => input
                            .into_par_iter()
                            .map(m)
                            .filter(f)
                            .map(l_m2)
                            .filter(f2)
                            .reduce(|| 0, l_r),
                    })
                })
            }
            Method::RayonRedWith => {
                let input = input.as_slice();
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| match h {
                    true => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .reduce_with(h_r),
                    false => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .reduce_with(l_r),
                })
            }
            Method::Orx => {
                let input = input.as_slice();
                match h {
                    true => input
                        .into_par()
                        .num_threads(input_variant.num_threads)
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .reduce(h_r),
                    false => input
                        .into_par()
                        .num_threads(input_variant.num_threads)
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .reduce(l_r),
                }
            }
            Method::OrxFixed => {
                let input = input.as_slice();
                match h {
                    true => input
                        .into_par()
                        .runner(Runner::fixed(Pool::default(input_variant.num_threads)))
                        .num_threads(input_variant.num_threads)
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .filter(f2)
                        .reduce(h_r),
                    false => input
                        .into_par()
                        .runner(Runner::fixed(Pool::default(input_variant.num_threads)))
                        .num_threads(input_variant.num_threads)
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .filter(f2)
                        .reduce(l_r),
                }
            }
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(match input_variant.heavy {
            true => input
                .iter()
                .map(m)
                .filter(f)
                .map(h_m2)
                .filter(f2)
                .reduce(h_r),
            false => input
                .iter()
                .map(m)
                .filter(f)
                .map(l_m2)
                .filter(f2)
                .reduce(l_r),
        })
    }
}

fn run(c: &mut Criterion) {
    let num_threads_options = [16, 32];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant {
                    n: 15,
                    heavy: false,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    heavy: false,
                    num_threads,
                },
                InputVariant {
                    n: 15,
                    heavy: true,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    heavy: true,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "reduce_mfmf", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
