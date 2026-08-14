use crate::{alg::Method, input::InputVariant};
use bench_helper::runner::fib;
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

const FIB_UPPER_BOUND: u64 = 20000;

pub struct Exp;

fn heterogeneous_map(heterogeneity_level: f64, i: u64) -> u64 {
    let mut rng = ChaCha8Rng::seed_from_u64(i);
    for _ in 0..10 * i {
        let _: u32 = rng.random();
    }

    let n = match rng.random_bool(heterogeneity_level) {
        true => rng.random_range(10000..20000),
        false => rng.random_range(1..100),
    };

    fib(FIB_UPPER_BOUND, n)
}

fn max_seq(input: &[u64], h: f64) -> Option<u64> {
    input.iter().map(|x| heterogeneous_map(h, *x)).max()
}

fn max_rayon(input: &[u64], h: f64) -> Option<u64> {
    input.par_iter().map(|x| heterogeneous_map(h, *x)).max()
}

fn max_orx(input: &[u64], h: f64) -> Option<u64> {
    input.par().map(|x| heterogeneous_map(h, *x)).max()
}

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
            Method::Seq => max_seq(input, h),
            Method::Rayon => max_rayon(input, h),
            Method::OrxOnce => max_orx(input, h),
            Method::OrxBasic => max_orx(input, h),
            Method::OrxRayon => max_orx(input, h),
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        let h = input_variant.heterogeneity_level;
        Some(max_seq(input, h))
    }
}
