use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

fn h_i(x: &u64, value: u64) -> Option<u64> {
    let y = match *x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    };
    (y == value).then_some(2 * y + 7 + x)
}

fn l_i(x: &u64, value: u64) -> Option<u64> {
    let y = match *x {
        999 => 999,
        n => 7 * n + 1000,
    };
    (y == value).then_some(2 * y + 7 + x)
}

#[derive(Debug, Clone, Copy)]
enum Pos {
    Beg,
    Mid,
    End,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    heavy: bool,
    pos: Pos,
    num_threads: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "pos", "task", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.pos {
                Pos::Beg => "beg",
                Pos::Mid => "mid",
                Pos::End => "end",
            }
            .to_string(),
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
                Self::Orx => "orx",
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
        let pos = match input_variant.pos {
            Pos::Beg => len / 20,
            Pos::Mid => len / 2,
            Pos::End => 19 * len / 20,
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
            Method::Rayon => {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| match input_variant.heavy {
                    false => input
                        .as_slice()
                        .into_par_iter()
                        .filter_map(|x| l_i(x, 999))
                        .find_first(|_| true),
                    true => input
                        .as_slice()
                        .into_par_iter()
                        .filter_map(|x| h_i(x, 999))
                        .find_first(|_| true),
                })
            }
            Method::Orx => match input_variant.heavy {
                false => input
                    .as_slice()
                    .into_par()
                    .num_threads(input_variant.num_threads)
                    .filter_map(|x| l_i(x, 999))
                    .first(),
                true => input
                    .as_slice()
                    .into_par()
                    .num_threads(input_variant.num_threads)
                    .filter_map(|x| h_i(x, 999))
                    .first(),
            },
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(match input_variant.heavy {
            false => input.iter().filter_map(|x| l_i(x, 999)).next(),
            true => input.iter().filter_map(|x| h_i(x, 999)).next(),
        })
    }

    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}
}

fn run(c: &mut Criterion) {
    let num_threads_options = [16, 32];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant {
                    n: 20,
                    pos: Pos::Beg,
                    heavy: false,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Mid,
                    heavy: false,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::End,
                    heavy: false,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Beg,
                    heavy: true,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Mid,
                    heavy: true,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::End,
                    heavy: true,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "first_i", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
