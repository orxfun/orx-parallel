use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

const FIB_UPPER_BOUND: u64 = 501;

fn inputs(len: usize, pos: usize, val: u64) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut vec = Vec::with_capacity(len);
    vec.extend((0..(len - 1)).map(|_| rng.random_range(0..150)));
    vec.insert(pos, val);
    vec
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

fn l_f(x: u64, value: u64) -> bool {
    x == value
}

fn h_f(x: u64, value: u64) -> bool {
    let a = black_box(fibonacci(x % FIB_UPPER_BOUND));
    let b = black_box(fibonacci(x % FIB_UPPER_BOUND));
    a - b + x == value
}

#[derive(Debug, Clone, Copy)]
enum Pos {
    Early,
    Mid,
    Late,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    heavy: bool,
    pos: Pos,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "pos", "task"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.pos {
                Pos::Early => "early",
                Pos::Mid => "mid",
                Pos::Late => "late",
            }
            .to_string(),
            match self.heavy {
                true => "heavy",
                false => "light",
            }
            .to_string(),
        ]
    }
}

enum Method {
    Seq,
    Rayon { nt: usize },
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

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let len = 1 << input_variant.n;
        let pos = match input_variant.pos {
            Pos::Early => 1 << 8,
            Pos::Mid => (1 << 19) + 7,
            Pos::Late => (1 << 20) - 27,
        };

        inputs(len, pos, 999)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => self.expected_output(input_variant, input).unwrap(),
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| match input_variant.heavy {
                    true => input
                        .as_slice()
                        .into_par_iter()
                        .filter(|x| x.is_multiple_of(9))
                        .filter(|x| h_f(**x, 999))
                        .find_first(|_| true)
                        .copied(),
                    false => input
                        .as_slice()
                        .into_par_iter()
                        .filter(|x| x.is_multiple_of(9))
                        .filter(|x| l_f(**x, 999))
                        .find_first(|_| true)
                        .copied(),
                })
            }
            Method::Orx { nt } => match input_variant.heavy {
                true => input
                    .as_slice()
                    .into_par()
                    .num_threads(*nt)
                    .filter(|x| x.is_multiple_of(9))
                    .filter(|x| h_f(**x, 999))
                    .first()
                    .copied(),
                false => input
                    .as_slice()
                    .into_par()
                    .num_threads(*nt)
                    .filter(|x| x.is_multiple_of(9))
                    .filter(|x| l_f(**x, 999))
                    .first()
                    .copied(),
            },
            Method::OrxFixed { nt } => match input_variant.heavy {
                true => input
                    .as_slice()
                    .into_par()
                    .runner(Runner::fixed(pool::get_global_pool()))
                    .num_threads(*nt)
                    .filter(|x| x.is_multiple_of(9))
                    .filter(|x| h_f(**x, 999))
                    .first()
                    .copied(),
                false => input
                    .as_slice()
                    .into_par()
                    .runner(Runner::fixed(pool::get_global_pool()))
                    .num_threads(*nt)
                    .filter(|x| x.is_multiple_of(9))
                    .filter(|x| l_f(**x, 999))
                    .first()
                    .copied(),
            },
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
                .find(|x| x.is_multiple_of(9) && h_f(**x, 999))
                .copied(),
            false => input
                .iter()
                .find(|x| x.is_multiple_of(9) && l_f(**x, 999))
                .copied(),
        })
    }

    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}
}

fn run(c: &mut Criterion) {
    let treatments: Vec<_> = vec![
        InputVariant {
            n: 20,
            pos: Pos::Early,
            heavy: false,
        },
        InputVariant {
            n: 20,
            pos: Pos::Mid,
            heavy: false,
        },
        InputVariant {
            n: 20,
            pos: Pos::Late,
            heavy: false,
        },
        InputVariant {
            n: 20,
            pos: Pos::Early,
            heavy: true,
        },
        InputVariant {
            n: 20,
            pos: Pos::Mid,
            heavy: true,
        },
        InputVariant {
            n: 20,
            pos: Pos::Late,
            heavy: true,
        },
    ];

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::Orx { nt },
            Method::OrxFixed { nt },
        ]
    };
    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "first_ff", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
