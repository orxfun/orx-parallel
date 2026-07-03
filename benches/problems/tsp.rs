use core::cmp::Ordering::Equal;
use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rayon::prelude::*;

const SEED: u64 = 42;

#[derive(Clone, Copy, Debug)]
struct Location {
    x: f64,
    y: f64,
}

impl Location {
    fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    fn tour_distance(locations: &[Location], tour: &[usize]) -> f64 {
        match (tour.first(), tour.last()) {
            (Some(&first), Some(&last)) => {
                let middle_distance: f64 = tour
                    .windows(2)
                    .map(|w| locations[w[0]].distance_to(locations[w[1]]))
                    .sum();
                let closing_distance = locations[last].distance_to(locations[first]);
                middle_distance + closing_distance
            }
            _ => 0.0,
        }
    }
}

fn locations(num_cities: usize) -> Vec<Location> {
    (0..num_cities).map(location_for).collect()
}

fn location_for(idx: usize) -> Location {
    let sx = split_mix_64((idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let sy = split_mix_64((idx as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));

    let x = 100.0 * to_unit_f64(sx) - 50.0;
    let y = 100.0 * to_unit_f64(sy) - 50.0;
    Location { x, y }
}

fn split_mix_64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn to_unit_f64(x: u64) -> f64 {
    let v = x >> 11;
    (v as f64) * (1.0 / ((1u64 << 53) as f64))
}

fn random_tour(seed: u64, num_cities: usize) -> Vec<usize> {
    let mut tour: Vec<usize> = (0..num_cities).collect();
    let mut rng = SmallRng::seed_from_u64(seed);
    tour.shuffle(&mut rng);
    tour
}

fn two_opt_improve(locations: &[Location], mut tour: Vec<usize>) -> Vec<usize> {
    let edge_distance = |i: usize, j: usize| locations[i].distance_to(locations[j]);

    let n = tour.len();
    if n < 4 {
        return tour;
    }

    let mut improved = true;
    while improved {
        improved = false;

        for i in 0..(n - 1) {
            let a = tour[i];
            let b = tour[(i + 1) % n];

            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }

                let c = tour[j];
                let d = tour[(j + 1) % n];

                let current = edge_distance(a, b) + edge_distance(c, d);
                let swapped = edge_distance(a, c) + edge_distance(b, d);

                if swapped + 1e-12 < current {
                    tour[(i + 1)..=j].reverse();
                    improved = true;
                    break;
                }
            }

            if improved {
                break;
            }
        }
    }

    tour
}

fn search_candidate(seed: u64, k: u64, locations: &[Location]) -> (Vec<usize>, f64) {
    let tour = random_tour(
        seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        locations.len(),
    );
    let tour = two_opt_improve(locations, tour);
    let distance = Location::tour_distance(locations, &tour);
    (tour, distance)
}

fn run_search_sequential(
    iterations: usize,
    seed: u64,
    locations: &[Location],
    start_index: u64,
) -> f64 {
    (0..iterations)
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), locations).1)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap_or(f64::INFINITY)
}

fn run_search_rayon(
    pool: &rayon::ThreadPool,
    iterations: usize,
    seed: u64,
    locations: &[Location],
    start_index: u64,
) -> f64 {
    pool.install(|| {
        (0..iterations)
            .into_par_iter()
            .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), locations).1)
            .reduce_with(|a, b| if a <= b { a } else { b })
            .unwrap_or(f64::INFINITY)
    })
}

fn run_search_orx<P: ParThreadPool>(
    pool: P,
    iterations: usize,
    seed: u64,
    num_threads: usize,
    chunk_size: usize,
    locations: &[Location],
    start_index: u64,
) -> f64 {
    (0..iterations)
        .into_par()
        .pool(pool)
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), locations).1)
        .reduce(|a, b| if a <= b { a } else { b })
        .unwrap_or(f64::INFINITY)
}

#[derive(Clone, Copy)]
struct Input {
    iterations: usize,
    num_cities: usize,
    num_threads: usize,
    chunk_size: usize,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["iter", "cities", "nt", "chunk"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.iterations.to_string(),
            self.num_cities.to_string(),
            self.num_threads.to_string(),
            self.chunk_size.to_string(),
        ]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["it", "c", "nt", "ch"]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
    OrxOnce,
    OrxBasic,
    OrxRayonCore,
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
                Self::OrxOnce => "orx-once",
                Self::OrxBasic => "orx-basic",
                Self::OrxRayonCore => "orx-rayon",
            }
            .to_string(),
        ]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["m"]
    }
}

struct BenchInput {
    locations: Vec<Location>,
    basic_pool: BasicPool,
    orx_rayon_pool: rayon_core::ThreadPool,
    rayon_pool: rayon::ThreadPool,
}

struct Exp;

impl Experiment for Exp {
    type InputFactors = Input;

    type AlgFactors = Method;

    type Input = BenchInput;

    type Output = f64;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let locations = locations(input_variant.num_cities);
        let basic_pool = Pool::basic(input_variant.num_threads);
        let orx_rayon_pool =
            Pool::rayon(input_variant.num_threads).expect("failed to create orx rayon-core pool");
        let rayon_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(input_variant.num_threads)
            .build()
            .expect("failed to create rayon pool");

        BenchInput {
            locations,
            basic_pool,
            orx_rayon_pool,
            rayon_pool,
        }
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let iterations = input_variant.iterations;
        let num_threads = input_variant.num_threads;
        let chunk_size = input_variant.chunk_size;
        let start_index = 0;

        match alg_variant {
            Method::Seq => run_search_sequential(iterations, SEED, &input.locations, start_index),
            Method::Rayon => run_search_rayon(
                &input.rayon_pool,
                iterations,
                SEED,
                &input.locations,
                start_index,
            ),
            Method::OrxOnce => run_search_orx(
                Pool::once(num_threads),
                iterations,
                SEED,
                num_threads,
                chunk_size,
                &input.locations,
                start_index,
            ),
            Method::OrxBasic => run_search_orx(
                &input.basic_pool,
                iterations,
                SEED,
                num_threads,
                chunk_size,
                &input.locations,
                start_index,
            ),
            Method::OrxRayonCore => run_search_orx(
                &input.orx_rayon_pool,
                iterations,
                SEED,
                num_threads,
                chunk_size,
                &input.locations,
                start_index,
            ),
        }
    }

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let expected = run_search_sequential(input_variant.iterations, SEED, &input.locations, 0);
        let diff = (expected - output).abs();
        assert!(
            diff <= 1e-9,
            "unexpected output diff={diff}, expected={expected}, got={output}"
        );
    }
}

fn run(c: &mut Criterion) {
    let chunk_sizes = [0usize, 1, 2, 4, 8, 16, 32, 64, 128, 256];

    let treatments: Vec<_> = chunk_sizes
        .into_iter()
        .map(|chunk_size| Input {
            iterations: 500,
            num_cities: 50,
            num_threads: 16,
            chunk_size,
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "problem_tsp", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
