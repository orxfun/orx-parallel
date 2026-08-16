use crate::{alg::Method, input::InputVariant};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

const CPU_MIX_ROUNDS: usize = 1000;

/// Represents a knapsack item with weight and value
#[derive(Clone, Debug)]
pub struct Item {
    pub weight: u32,
    pub value: u32,
}

/// Represents a single solution (chromosome) in the knapsack problem
#[derive(Clone, Debug)]
pub struct Individual {
    pub genes: Vec<bool>, // true = item included, false = not included
    pub fitness: u32,
}

impl Individual {
    pub fn new(genes: Vec<bool>) -> Self {
        Individual { genes, fitness: 0 }
    }

    pub fn evaluate(&mut self, items: &[Item], capacity: u32) {
        let mut total_weight = 0u32;
        let mut total_value = 0u32;

        let _ = black_box(cpu_mix(CPU_MIX_ROUNDS, total_value as u64));

        for (i, &included) in self.genes.iter().enumerate() {
            if included {
                total_weight += items[i].weight;
                total_value += items[i].value;
            }
        }

        if total_weight > capacity {
            // Penalty for exceeding capacity
            self.fitness = 0;
        } else {
            self.fitness = total_value;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaResult {
    pub best_fitness: u32,
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

fn knapsack_capacity(items: &[Item]) -> u32 {
    let total_weight: u32 = items.iter().map(|item| item.weight).sum();
    (total_weight as f64 * 0.6) as u32 // Capacity is 60% of total weight
}

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = (Vec<Item>, u32, usize);
    type Output = GaResult;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        const SEED: u64 = 54321;
        let items = create_items(SEED, input_variant.num_items);
        let capacity = knapsack_capacity(&items);
        let generations = 50;
        (items, capacity, generations)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (items, capacity, generations) = input;
        let population_size = input_variant.population_size;
        match alg_variant {
            Method::Seq => run_ga_sequential(items, *capacity, *generations, population_size),
            Method::Rayon => run_ga_rayon(items, *capacity, *generations, population_size),
            Method::OrxOnce => run_ga_orx(items, *capacity, *generations, population_size),
            Method::OrxBasic => run_ga_orx(items, *capacity, *generations, population_size),
            Method::OrxRayon => run_ga_orx(items, *capacity, *generations, population_size),
        }
    }

    fn validate_output(
        &self,
        _input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let (items, capacity, _) = input;

        // Validate weight doesn't exceed capacity
        assert!(
            output.total_weight <= *capacity,
            "Total weight {} exceeds capacity {}",
            output.total_weight,
            *capacity
        );

        // Validate weight and fitness are plausible
        let max_possible_value: u32 = items.iter().map(|i| i.value).sum();
        assert!(
            output.best_fitness <= max_possible_value,
            "Fitness {} exceeds maximum possible value {}",
            output.best_fitness,
            max_possible_value
        );
    }
}

fn run_ga_sequential(
    items: &[Item],
    capacity: u32,
    generations: usize,
    population_size: usize,
) -> GaResult {
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let mut population = create_initial_population(&mut rng, items.len(), population_size);

    for _ in 0..generations {
        // Evaluate fitness
        for individual in &mut population {
            individual.evaluate(items, capacity);
        }

        // Sort by fitness (descending)
        population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        // Keep top 50%, breed new population
        let num_survivors = population_size / 2;
        let survivors = population[0..num_survivors].to_vec();

        population = survivors.clone();
        for _ in num_survivors..population_size {
            let parent1 = &survivors[rng.random_range(0..survivors.len())];
            let parent2 = &survivors[rng.random_range(0..survivors.len())];
            let child = crossover(parent1, parent2, &mut rng);
            population.push(child);
        }
    }

    // Final evaluation
    for individual in &mut population {
        individual.evaluate(items, capacity);
    }
    population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

    let best = &population[0];
    let total_weight = best
        .genes
        .iter()
        .enumerate()
        .filter(|(_, &included)| included)
        .map(|(i, _)| items[i].weight)
        .sum();

    GaResult {
        best_fitness: best.fitness,
        total_weight,
    }
}

fn run_ga_rayon(
    items: &[Item],
    capacity: u32,
    generations: usize,
    population_size: usize,
) -> GaResult {
    use rayon::prelude::*;

    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let mut population = create_initial_population(&mut rng, items.len(), population_size);

    for _ in 0..generations {
        // Parallel fitness evaluation
        population.par_iter_mut().for_each(|individual| {
            individual.evaluate(items, capacity);
        });

        // Sort by fitness (descending)
        population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        // Keep top 50%, breed new population in parallel
        let num_survivors = population_size / 2;
        let survivors = population[0..num_survivors].to_vec();

        population = survivors.clone();

        // Parallel generation of offspring
        let offspring: Vec<_> = (0..(population_size - num_survivors))
            .into_par_iter()
            .map_with(ChaCha8Rng::seed_from_u64(rng.random()), |rng, _| {
                let parent1 = &survivors[rng.random_range(0..survivors.len())];
                let parent2 = &survivors[rng.random_range(0..survivors.len())];
                crossover(parent1, parent2, rng)
            })
            .collect();

        population.extend(offspring);
    }

    // Final parallel evaluation
    population.par_iter_mut().for_each(|individual| {
        individual.evaluate(items, capacity);
    });
    population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

    let best = &population[0];
    let total_weight = best
        .genes
        .iter()
        .enumerate()
        .filter(|(_, &included)| included)
        .map(|(i, _)| items[i].weight)
        .sum();

    GaResult {
        best_fitness: best.fitness,
        total_weight,
    }
}

fn run_ga_orx(
    items: &[Item],
    capacity: u32,
    generations: usize,
    population_size: usize,
) -> GaResult {
    use orx_parallel::*;

    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let mut population = create_initial_population(&mut rng, items.len(), population_size);

    for _ in 0..generations {
        // Parallel fitness evaluation
        let population_vec: Vec<_> = (0..population.len())
            .par()
            .map(|i| {
                let mut individual = population[i].clone();
                individual.evaluate(items, capacity);
                individual
            })
            .collect();
        population = population_vec;

        // Sort by fitness (descending)
        population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        // Keep top 50%, breed new population in parallel
        let num_survivors = population_size / 2;
        let survivors = population[0..num_survivors].to_vec();

        population = survivors.clone();

        // Parallel generation of offspring
        let offspring: Vec<_> = (0..(population_size - num_survivors))
            .par()
            .use_new(|i| ChaCha8Rng::seed_from_u64(i as u64))
            .map(|rng, _| {
                let parent1 = &survivors[rng.random_range(0..survivors.len())];
                let parent2 = &survivors[rng.random_range(0..survivors.len())];
                crossover(parent1, parent2, rng)
            })
            .collect();

        population.extend(offspring);
    }

    // Final parallel evaluation
    let population_vec: Vec<_> = (0..population.len())
        .par()
        .map(|i| {
            let mut individual = population[i].clone();
            individual.evaluate(items, capacity);
            individual
        })
        .collect();
    population = population_vec;

    population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

    let best = &population[0];
    let total_weight = best
        .genes
        .iter()
        .enumerate()
        .filter(|(_, &included)| included)
        .map(|(i, _)| items[i].weight)
        .sum();

    GaResult {
        best_fitness: best.fitness,
        total_weight,
    }
}

fn create_initial_population(
    rng: &mut impl Rng,
    num_items: usize,
    population_size: usize,
) -> Vec<Individual> {
    (0..population_size)
        .map(|_| {
            let genes = (0..num_items)
                .map(|_| rng.random_range(0..2) == 1)
                .collect();
            Individual::new(genes)
        })
        .collect()
}

fn crossover(parent1: &Individual, parent2: &Individual, rng: &mut impl Rng) -> Individual {
    let crossover_point = rng.random_range(0..parent1.genes.len());
    let mut child_genes = parent1.genes[0..crossover_point].to_vec();
    child_genes.extend_from_slice(&parent2.genes[crossover_point..]);

    // Mutation (2% mutation rate)
    for gene in &mut child_genes {
        if rng.random_range(0..100) < 2 {
            *gene = !*gene;
        }
    }

    Individual::new(child_genes)
}
