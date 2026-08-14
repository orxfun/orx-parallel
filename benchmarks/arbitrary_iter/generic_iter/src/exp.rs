use crate::{alg::Method, input::InputVariant};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

const CPU_MIX_ROUNDS: usize = 10000;

pub struct Exp;

#[derive(Clone, Debug)]
pub struct Input {
    vec: Vec<u64>,
}

impl Input {
    fn get_iter(&self) -> impl Iterator<Item = u64> {
        self.vec
            .iter()
            .copied()
            .map(|x| black_box(x + 1))
            .filter(|x| !x.is_multiple_of(7))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Agg {
    sum: u64,
    count: u64,
}

impl Agg {
    fn from_val(v: u64) -> Self {
        Self { sum: v, count: 1 }
    }
}

fn merge(a: Agg, b: Agg) -> Agg {
    Agg {
        sum: a.sum.wrapping_add(b.sum),
        count: a.count + b.count,
    }
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Input;

    type Output = Agg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let len = input_variant.n;
        const SEED: u64 = 0xA5A5_4F4F_0101_BEEF;
        let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

        let vec = (0..len).map(|_| rng.random_range(1..=1_000_000)).collect();
        Input { vec }
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let iter = input.get_iter();
        match alg_variant {
            Method::Seq => seq_set(iter),
            Method::Rayon => rayon_set(iter),
            Method::OrxOnce => orx_set(iter),
            Method::OrxBasic => orx_set(iter),
            Method::OrxRayon => orx_set(iter),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(seq_set(input.get_iter()))
    }
}

fn keep(v: u64) -> bool {
    v & 0x7 != 0 && !v.trailing_zeros().is_multiple_of(3)
}

fn seq_set(iter: impl Iterator<Item = u64>) -> Agg {
    iter.map(|k| cpu_mix(CPU_MIX_ROUNDS, k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn rayon_set<I>(iter: I) -> Agg
where
    I: rayon::iter::ParallelBridge,
    I: Iterator<Item = u64>,
    I: Send,
{
    use rayon::prelude::*;
    iter.par_bridge()
        .map(|k| cpu_mix(CPU_MIX_ROUNDS, k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(Agg::default, merge)
}

fn orx_set(iter: impl Iterator<Item = u64>) -> Agg {
    use orx_parallel::*;
    iter.iter_into_par()
        .map(|k| cpu_mix(CPU_MIX_ROUNDS, k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}
