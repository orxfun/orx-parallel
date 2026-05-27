use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
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

fn h_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    }
}

fn l_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

#[derive(Debug, Clone, Copy)]
enum Pos {
    Beg,
    Mid,
    End,
}

#[derive(Clone, Copy)]
struct Input {
    n: usize,
    heavy: bool,
    pos: Pos,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "pos", "task"]
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
        ]
    }
}

#[derive(Debug, Sequence, Clone, Copy)]
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
    type InputFactors = Input;

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
                let iter = input.as_slice().into_par_iter();
                match input_variant.heavy {
                    false => iter.map(l_m).filter(|x| *x == 999).find_first(|_| true),
                    true => iter.map(h_m).filter(|x| *x == 999).find_first(|_| true),
                }
            }
            Method::Orx => match input_variant.heavy {
                false => input
                    .as_slice()
                    .into_par()
                    .map(l_m)
                    .filter(|x| *x == 999)
                    .first(),
                true => input
                    .as_slice()
                    .into_par()
                    .map(h_m)
                    .filter(|x| *x == 999)
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
            false => input.iter().map(l_m).filter(|x| *x == 999).next(),
            true => input.iter().map(h_m).filter(|x| *x == 999).next(),
        })
    }

    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}
}

fn run(c: &mut Criterion) {
    let treatments = [
        Input {
            n: 20,
            pos: Pos::Beg,
            heavy: false,
        },
        Input {
            n: 20,
            pos: Pos::Mid,
            heavy: false,
        },
        Input {
            n: 20,
            pos: Pos::End,
            heavy: false,
        },
        Input {
            n: 20,
            pos: Pos::Beg,
            heavy: true,
        },
        Input {
            n: 20,
            pos: Pos::Mid,
            heavy: true,
        },
        Input {
            n: 20,
            pos: Pos::End,
            heavy: true,
        },
    ];

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "first_mf", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
