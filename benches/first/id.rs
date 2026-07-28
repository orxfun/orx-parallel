use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    num_threads: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("2e{}", self.n), self.num_threads.to_string()]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
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
        inputs(1 << input_variant.n)
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
                pool.install(|| {
                    input
                        .as_slice()
                        .into_par_iter()
                        .find_first(|_| true)
                        .copied()
                })
            }
            Method::Orx => input
                .as_slice()
                .into_par()
                .num_threads(input_variant.num_threads)
                .first()
                .copied(),
            Method::OrxFixed => input
                .as_slice()
                .into_par()
                .runner(Runner::fixed(Pool::default(input_variant.num_threads)))
                .num_threads(input_variant.num_threads)
                .first()
                .copied(),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(input.iter().next().copied())
    }

    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}
}

fn run(c: &mut Criterion) {
    let num_threads_options = [16, 32];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant { n: 10, num_threads },
                InputVariant { n: 15, num_threads },
                InputVariant { n: 20, num_threads },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "first_id", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
