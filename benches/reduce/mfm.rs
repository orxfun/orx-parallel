use criterion::{Criterion, criterion_group, criterion_main};
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
        const SEED: u64 = 654;
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let len = 1 << input_variant.n;
        (0..len).map(|_| rng.random_range(0..150)).collect()
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
                true => input.iter().map(m).filter(f).map(h_m2).reduce(h_r),
                false => input.iter().map(m).filter(f).map(l_m2).reduce(l_r),
            },
            Method::RayonRedWith { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| match h {
                    true => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(h_m2)
                        .reduce_with(h_r),
                    false => input
                        .into_par_iter()
                        .map(m)
                        .filter(f)
                        .map(l_m2)
                        .reduce_with(l_r),
                })
            }
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| {
                    Some(match h {
                        true => input
                            .into_par_iter()
                            .map(m)
                            .filter(f)
                            .map(h_m2)
                            .reduce(|| 0, h_r),
                        false => input
                            .into_par_iter()
                            .map(m)
                            .filter(f)
                            .map(l_m2)
                            .reduce(|| 0, l_r),
                    })
                })
            }
            Method::Orx { nt } => match h {
                true => input
                    .into_par()
                    .num_threads(*nt)
                    .map(m)
                    .filter(f)
                    .map(h_m2)
                    .reduce(h_r),
                false => input
                    .into_par()
                    .num_threads(*nt)
                    .map(m)
                    .filter(f)
                    .map(l_m2)
                    .reduce(l_r),
            },
            Method::OrxFixed { nt } => match h {
                true => input
                    .into_par()
                    .runner(Runner::fixed())
                    .num_threads(*nt)
                    .map(m)
                    .filter(f)
                    .map(h_m2)
                    .reduce(h_r),
                false => input
                    .into_par()
                    .runner(Runner::fixed())
                    .num_threads(*nt)
                    .map(m)
                    .filter(f)
                    .map(l_m2)
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
            true => input.iter().map(m).filter(f).map(h_m2).reduce(h_r),
            false => input.iter().map(m).filter(f).map(l_m2).reduce(l_r),
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

    Exp.bench(c, "reduce_mfm", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
