use criterion::{Criterion, criterion_group, criterion_main};
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
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "het-lvl"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            format!("{:4}", self.heterogeneity_level),
        ]
    }
}

#[derive(Debug)]
enum Method {
    Seq,
    Rayon { nt: usize },
    Orx { nt: usize },
    OrxFix { nt: usize },
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
            Self::OrxFix { nt } => format!("orx-fix-{nt}"),
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
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| input.par_iter().map(|x| heterogeneous_map(h, *x)).max())
            }
            Method::Orx { nt } => input
                .par()
                .num_threads(*nt)
                .map(|x| heterogeneous_map(h, *x))
                .max(),
            Method::OrxFix { nt } => input
                .par()
                .runner(Runner::fixed())
                .num_threads(*nt)
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
    let ns = [10, 12, 14];
    let heterogeneity_levels = [0.001, 0.011, 0.101, 0.201];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| {
            heterogeneity_levels.map(|heterogeneity_level| InputVariant {
                n,
                heterogeneity_level,
            })
        })
        .collect();

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::Orx { nt },
            Method::OrxFix { nt },
        ]
    };

    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "het_simple", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
