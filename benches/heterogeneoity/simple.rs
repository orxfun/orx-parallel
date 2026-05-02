use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn heterogeneous_map(heterogeneoity_level: f64, i: u64) -> u64 {
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

    let mut rng = ChaCha8Rng::seed_from_u64(i);
    for _ in 0..10 * i {
        let _: u32 = rng.random();
    }

    let n = match rng.random_bool(heterogeneoity_level) {
        true => rng.random_range(10000..20000),
        false => rng.random_range(1..100),
    };

    fibonacci(n)
}

#[derive(Clone, Copy)]
struct Input {
    n: usize,
    heterogeneoity_level: f64,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "het-lvl"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            format!("{:4}", self.heterogeneoity_level),
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
    type InputFactors = Input;

    type AlgFactors = Method;

    type Input = (Input, Vec<u64>);

    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let len = 1 << input_variant.n;
        (*input_variant, (0..len).map(|i| i as u64).collect())
    }

    fn execute(&mut self, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        let h = input.0.heterogeneoity_level;
        match alg_variant {
            Method::Seq => self.expected_output(&input.0, input).unwrap(),
            Method::Rayon => input.1.par_iter().map(|x| heterogeneous_map(h, *x)).max(),
            Method::Orx => input.1.par().map(|x| heterogeneous_map(h, *x)).max(),
        }
    }

    fn expected_output(
        &self,
        _: &Self::InputFactors,
        (input_variant, input): &Self::Input,
    ) -> Option<Self::Output> {
        let h = input_variant.heterogeneoity_level;
        Some(input.iter().map(|x| heterogeneous_map(h, *x)).max())
    }
}

fn run(c: &mut Criterion) {
    let treatments = [
        Input {
            n: 10,
            heterogeneoity_level: 0.01,
        },
        Input {
            n: 15,
            heterogeneoity_level: 0.01,
        },
        Input {
            n: 10,
            heterogeneoity_level: 0.05,
        },
        Input {
            n: 15,
            heterogeneoity_level: 0.05,
        },
        Input {
            n: 10,
            heterogeneoity_level: 0.10,
        },
        Input {
            n: 15,
            heterogeneoity_level: 0.10,
        },
    ];

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "first_id", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
