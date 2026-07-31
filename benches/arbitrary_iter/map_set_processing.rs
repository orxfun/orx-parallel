//! Arbitrary-iterator benchmark for HashMap/HashSet processing via iterators.
//! Compares sequential iteration with rayon bridge and orx `iter_into_par`
//! pipelines over map/filter/reduce workloads.

use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
enum Dataset {
    Map,
    Set,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    dataset: Dataset,
    num_threads: usize,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "dataset", "nt"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.dataset {
                Dataset::Map => "hash-map",
                Dataset::Set => "hash-set",
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Agg {
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

#[derive(Clone, Debug)]
struct Input {
    map: HashMap<u64, u32>,
    set: HashSet<u64>,
}

struct Exp;

fn mix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn keep(v: u64) -> bool {
    v & 0x7 != 0 && !v.trailing_zeros().is_multiple_of(3)
}

fn inputs(len: usize) -> Input {
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

fn seq_map(map: &HashMap<u64, u32>) -> Agg {
    map.iter()
        .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn seq_set(set: &HashSet<u64>) -> Agg {
    set.iter()
        .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn rayon_map(map: &HashMap<u64, u32>, num_threads: usize) -> Agg {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        map.iter()
            .par_bridge()
            .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
            .filter(|x| keep(*x))
            .map(Agg::from_val)
            .reduce(Agg::default, merge)
    })
}

fn rayon_set(set: &HashSet<u64>, num_threads: usize) -> Agg {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        set.iter()
            .par_bridge()
            .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
            .filter(|x| keep(*x))
            .map(Agg::from_val)
            .reduce(Agg::default, merge)
    })
}

fn orx_map(map: &HashMap<u64, u32>, num_threads: usize) -> Agg {
    map.iter()
        .iter_into_par()
        .num_threads(num_threads)
        .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn orx_fixed_map(map: &HashMap<u64, u32>, num_threads: usize) -> Agg {
    map.iter()
        .iter_into_par()
        .runner(Runner::fixed(Pool::default(num_threads)))
        .num_threads(num_threads)
        .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn orx_set(set: &HashSet<u64>, num_threads: usize) -> Agg {
    set.iter()
        .iter_into_par()
        .num_threads(num_threads)
        .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn orx_fixed_set(set: &HashSet<u64>, num_threads: usize) -> Agg {
    set.iter()
        .iter_into_par()
        .runner(Runner::fixed(Pool::default(num_threads)))
        .num_threads(num_threads)
        .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Input;

    type Output = Agg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len())
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match (input_variant.dataset, alg_variant) {
            (Dataset::Map, Method::Seq) => seq_map(&input.map),
            (Dataset::Map, Method::Rayon) => rayon_map(&input.map, input_variant.num_threads),
            (Dataset::Map, Method::Orx) => orx_map(&input.map, input_variant.num_threads),
            (Dataset::Map, Method::OrxFixed) => {
                orx_fixed_map(&input.map, input_variant.num_threads)
            }

            (Dataset::Set, Method::Seq) => seq_set(&input.set),
            (Dataset::Set, Method::Rayon) => rayon_set(&input.set, input_variant.num_threads),
            (Dataset::Set, Method::Orx) => orx_set(&input.set, input_variant.num_threads),
            (Dataset::Set, Method::OrxFixed) => {
                orx_fixed_set(&input.set, input_variant.num_threads)
            }
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(match input_variant.dataset {
            Dataset::Map => seq_map(&input.map),
            Dataset::Set => seq_set(&input.set),
        })
    }
}

fn run(c: &mut Criterion) {
    let num_threads_options = [4, 16];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant {
                    n: 16,
                    dataset: Dataset::Map,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    dataset: Dataset::Map,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    dataset: Dataset::Set,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    dataset: Dataset::Set,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(
        c,
        "arbitrary_iter_map_set_processing",
        &treatments,
        &variants,
    );
}

criterion_group!(benches, run);
criterion_main!(benches);
