use crate::{alg::Method, input::InputVariant};
use orx_criterion::Experiment;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

const INITIAL_TEMPERATURE: f64 = 25.0;
const COOLING_RATE: f64 = 0.995;

#[derive(Clone, Debug)]
pub struct Item {
    pub weight: u32,
    pub value: u32,
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub genes: Vec<bool>,
    pub value: u32,
    pub weight: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaResult {
    pub best_value: u32,
    pub total_weight: u32,
}

fn create_items(seed: u64, num_items: usize) -> Vec<Item> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..num_items)
        .map(|_| Item {
            weight: rng.random_range(1..100),
            value: rng.random_range(1..100),
        })
        .collect()
}

fn capacity(items: &[Item]) -> u32 {
    items.iter().map(|item| item.weight).sum::<u32>() * 3 / 5
}

fn evaluate(genes: &[bool], items: &[Item]) -> (u32, u32) {
    genes
        .iter()
        .zip(items)
        .filter(|(included, _)| **included)
        .fold((0, 0), |(value, weight), (_, item)| {
            (value + item.value, weight + item.weight)
        })
}

fn initial_candidate(num_items: usize) -> Candidate {
    Candidate {
        genes: vec![false; num_items],
        value: 0,
        weight: 0,
    }
}

fn neighbor(current: &Candidate, items: &[Item], rng: &mut impl Rng) -> Candidate {
    let mut candidate = current.clone();
    let index = rng.random_range(0..candidate.genes.len());
    candidate.genes[index] = !candidate.genes[index];
    (candidate.value, candidate.weight) = evaluate(&candidate.genes, items);
    candidate
}

fn anneal(
    items: &[Item],
    capacity: u32,
    steps: usize,
    seed: u64,
    shared_best: Option<&AtomicU32>,
) -> Candidate {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut current = initial_candidate(items.len());
    let mut best = current.clone();
    let mut temperature = INITIAL_TEMPERATURE;

    for _ in 0..steps {
        let candidate = neighbor(&current, items, &mut rng);
        if candidate.weight <= capacity {
            let delta = candidate.value as f64 - current.value as f64;
            let accept =
                delta >= 0.0 || rng.random::<f64>() < (delta / temperature.max(0.001)).exp();
            if accept {
                current = candidate;
            }

            if current.value > best.value {
                best = current.clone();
                if let Some(shared_best) = shared_best {
                    shared_best.fetch_max(best.value, Ordering::Relaxed);
                }
            }
        }
        temperature *= COOLING_RATE;
    }

    best
}

fn result(candidate: Candidate) -> SaResult {
    SaResult {
        best_value: candidate.value,
        total_weight: candidate.weight,
    }
}

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = (Vec<Item>, u32);
    type Output = SaResult;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let items = create_items(0x5eed, input_variant.num_items);
        let capacity = capacity(&items);
        (items, capacity)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (items, capacity) = input;
        match alg_variant {
            Method::Seq => run_sequential(items, *capacity, input_variant),
            Method::Rayon => run_rayon(items, *capacity, input_variant),
            Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => {
                run_orx(items, *capacity, input_variant)
            }
        }
    }

    fn validate_output(
        &self,
        _input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let (items, capacity) = input;
        assert!(output.total_weight <= *capacity);

        let max_value = items.iter().map(|item| item.value).sum::<u32>();
        assert!(output.best_value <= max_value);
    }
}

fn run_sequential(items: &[Item], capacity: u32, input: &InputVariant) -> SaResult {
    let shared_best = AtomicU32::new(0);
    let best = (0..input.restarts)
        .map(|restart| {
            anneal(
                items,
                capacity,
                input.steps,
                restart as u64,
                Some(&shared_best),
            )
        })
        .max_by_key(|candidate| candidate.value)
        .expect("at least one restart");
    result(best)
}

fn run_rayon(items: &[Item], capacity: u32, input: &InputVariant) -> SaResult {
    use rayon::prelude::*;

    let shared_best = Arc::new(AtomicU32::new(0));
    let best = (0..input.restarts)
        .into_par_iter()
        .map(|restart| {
            anneal(
                items,
                capacity,
                input.steps,
                restart as u64,
                Some(shared_best.as_ref()),
            )
        })
        .max_by_key(|candidate| candidate.value)
        .expect("at least one restart");
    result(best)
}

fn run_orx(items: &[Item], capacity: u32, input: &InputVariant) -> SaResult {
    use orx_parallel::*;

    let shared_best = Arc::new(AtomicU32::new(0));
    let best = (0..input.restarts)
        .into_par()
        .map(|restart| {
            anneal(
                items,
                capacity,
                input.steps,
                restart as u64,
                Some(shared_best.as_ref()),
            )
        })
        .max_by_key(|candidate| candidate.value)
        .expect("at least one restart");
    result(best)
}
