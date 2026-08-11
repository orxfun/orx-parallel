use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

const FIB_UPPER_BOUND: u64 = 501;

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
struct InputVariant {
    n: usize,
    heavy: bool,
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

#[derive(Debug)]
enum Method {
    Seq,
    Rayon { nt: usize },
    RayonRedWith { nt: usize },
    Orx { nt: usize },
    OrxFixed { nt: usize },
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon { nt } => format!("rayon-{nt}"),
            Self::RayonRedWith { nt } => format!("rayon-red-with-{nt}"),
            Self::Orx { nt } => format!("orx-{nt}"),
            Self::OrxFixed { nt } => format!("orx-fixed-{nt}"),
        }]
    }
}

struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<u64>;

    type Output = Option<u64>;

    type GroupArtifact = ();

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(1 << input_variant.n)
    }

    fn group_artifact(
        &mut self,
        _: &Self::InputFactors,
        _: &Self::AlgFactors,
        _: &Self::Input,
    ) -> Self::GroupArtifact {
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
        _: &mut Self::GroupArtifact,
    ) -> Self::Output {
        let h = input_variant.heavy;
        let input = input.as_slice();
        match alg_variant {
            Method::Seq => match h {
                true => input.iter().flat_map(l_l).reduce(h_r),
                false => input.iter().flat_map(h_l).reduce(l_r),
            },
            Method::RayonRedWith { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| match h {
                    true => input.into_par_iter().flat_map_iter(l_l).reduce_with(h_r),
                    false => input.into_par_iter().flat_map_iter(h_l).reduce_with(l_r),
                })
            }
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| {
                    Some(match h {
                        true => input.into_par_iter().flat_map_iter(l_l).reduce(|| 0, h_r),
                        false => input.into_par_iter().flat_map_iter(h_l).reduce(|| 0, l_r),
                    })
                })
            }
            Method::Orx { nt } => match h {
                true => input.into_par().num_threads(*nt).flat_map(l_l).reduce(h_r),
                false => input.into_par().num_threads(*nt).flat_map(h_l).reduce(l_r),
            },
            Method::OrxFixed { nt } => match h {
                true => input
                    .into_par()
                    .runner(Runner::fixed())
                    .num_threads(*nt)
                    .flat_map(l_l)
                    .reduce(h_r),
                false => input
                    .into_par()
                    .runner(Runner::fixed())
                    .num_threads(*nt)
                    .flat_map(h_l)
                    .reduce(l_r),
            },
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(match input_variant.heavy {
            true => input.iter().flat_map(l_l).reduce(h_r),
            false => input.iter().flat_map(h_l).reduce(l_r),
        })
    }
}

fn run(c: &mut Criterion) {
    let treatments: Vec<_> = vec![
        InputVariant {
            n: 15,
            heavy: false,
        },
        InputVariant {
            n: 20,
            heavy: false,
        },
        InputVariant { n: 15, heavy: true },
        InputVariant { n: 20, heavy: true },
    ];

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::RayonRedWith { nt },
            Method::Orx { nt },
            Method::OrxFixed { nt },
        ]
    };
    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "reduce_l", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
