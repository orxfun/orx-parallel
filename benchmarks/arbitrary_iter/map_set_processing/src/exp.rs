use crate::input::Dataset;
use crate::{alg::Method, input::InputVariant};
use orx_criterion::Experiment;
use orx_parallel_bench_helper::runner::cpu_mix;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};

const CPU_MIX_ROUNDS: usize = 10000;

pub struct Exp;

#[derive(Clone, Debug)]
pub struct Input {
    map: HashMap<u64, u32>,
    set: HashSet<u64>,
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

        let mut map = HashMap::with_capacity(len);
        let mut set = HashSet::with_capacity(len);

        for idx in 0..len {
            let key_map = (idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x1234_5678_90AB_CDEF;
            let value = rng.random_range(1..=1_000_000);
            map.insert(key_map, value);

            let key_set = (idx as u64).wrapping_mul(0xD6E8_FD9D_79A1_4E3B) ^ 0x0FED_CBA9_8765_4321;
            set.insert(key_set);
        }

        Input { map, set }
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match input_variant.dataset {
            Dataset::Map => match alg_variant {
                Method::Seq => seq_map(&input.map),
                Method::Rayon => rayon_map(&input.map),
                Method::OrxOnce => orx_map(&input.map),
                Method::OrxBasic => orx_map(&input.map),
                Method::OrxRayon => orx_map(&input.map),
            },
            Dataset::Set => match alg_variant {
                Method::Seq => seq_set(&input.set),
                Method::Rayon => rayon_set(&input.set),
                Method::OrxOnce => orx_set(&input.set),
                Method::OrxBasic => orx_set(&input.set),
                Method::OrxRayon => orx_set(&input.set),
            },
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        match input_variant.dataset {
            Dataset::Map => Some(seq_map(&input.map)),
            Dataset::Set => Some(seq_set(&input.set)),
        }
    }
}

fn keep(v: u64) -> bool {
    v & 0x7 != 0 && !v.trailing_zeros().is_multiple_of(3)
}

fn seq_map(map: &HashMap<u64, u32>) -> Agg {
    map.iter()
        .map(|(k, v)| cpu_mix(CPU_MIX_ROUNDS, *k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn seq_set(set: &HashSet<u64>) -> Agg {
    set.iter()
        .map(|k| cpu_mix(CPU_MIX_ROUNDS, *k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn rayon_map(map: &HashMap<u64, u32>) -> Agg {
    use rayon::prelude::*;
    map.iter()
        .par_bridge()
        .map(|(k, v)| cpu_mix(CPU_MIX_ROUNDS, *k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(Agg::default, merge)
}

fn rayon_set(set: &HashSet<u64>) -> Agg {
    use rayon::prelude::*;
    set.iter()
        .par_bridge()
        .map(|k| cpu_mix(CPU_MIX_ROUNDS, *k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(Agg::default, merge)
}

fn orx_map(map: &HashMap<u64, u32>) -> Agg {
    use orx_parallel::*;
    map.iter()
        .iter_into_par()
        .map(|(k, v)| cpu_mix(CPU_MIX_ROUNDS, *k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn orx_set(set: &HashSet<u64>) -> Agg {
    use orx_parallel::*;
    set.iter()
        .iter_into_par()
        .map(|k| cpu_mix(CPU_MIX_ROUNDS, *k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}
