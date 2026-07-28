use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn heterogeneous_map(heterogeneity_level: f64, i: u64) -> u64 {
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

    let n = match rng.random_bool(heterogeneity_level) {
        true => rng.random_range(10000..20000),
        false => rng.random_range(1..100),
    };

    fibonacci(n)
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    heterogeneity_level: f64,
    num_threads: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "het-lvl", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            format!("{:4}", self.heterogeneity_level),
            self.num_threads.to_string(),
        ]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
    OrxFix,
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
                Self::OrxFix => "orx-fix",
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
        (0..len).map(|i| i as u64).collect()
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let h = input_variant.heterogeneity_level;
        match alg_variant {
            Method::Seq => self.expected_output(input_variant, input).unwrap(),
            Method::Rayon => {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| input.par_iter().map(|x| heterogeneous_map(h, *x)).max())
            }
            Method::OrxFix => input
                .par()
                .runner(Runner::fixed(Pool::once(input_variant.num_threads)))
                .map(|x| heterogeneous_map(h, *x))
                .max(),
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        let h = input_variant.heterogeneity_level;
        Some(input.iter().map(|x| heterogeneous_map(h, *x)).max())
    }
}

fn run(c: &mut Criterion) {
    let num_threads_options = [16, 32];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant {
                    n: 10,
                    heterogeneity_level: 0.001,
                    num_threads,
                },
                InputVariant {
                    n: 10,
                    heterogeneity_level: 0.011,
                    num_threads,
                },
                InputVariant {
                    n: 10,
                    heterogeneity_level: 0.101,
                    num_threads,
                },
                InputVariant {
                    n: 10,
                    heterogeneity_level: 0.201,
                    num_threads,
                },
                InputVariant {
                    n: 12,
                    heterogeneity_level: 0.001,
                    num_threads,
                },
                InputVariant {
                    n: 12,
                    heterogeneity_level: 0.011,
                    num_threads,
                },
                InputVariant {
                    n: 12,
                    heterogeneity_level: 0.101,
                    num_threads,
                },
                InputVariant {
                    n: 12,
                    heterogeneity_level: 0.201,
                    num_threads,
                },
                InputVariant {
                    n: 14,
                    heterogeneity_level: 0.001,
                    num_threads,
                },
                InputVariant {
                    n: 14,
                    heterogeneity_level: 0.011,
                    num_threads,
                },
                InputVariant {
                    n: 14,
                    heterogeneity_level: 0.101,
                    num_threads,
                },
                InputVariant {
                    n: 14,
                    heterogeneity_level: 0.201,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "het_simple", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
