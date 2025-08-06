use orx_parallel::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn random_walk(rng: &mut impl Rng, position: i64, num_steps: usize) -> i64 {
    (0..num_steps).fold(position, |p, _| random_step(rng, p))
}

fn random_step(rng: &mut impl Rng, position: i64) -> i64 {
    match rng.random_bool(0.5) {
        true => position + 1,  // to right
        false => position - 1, // to left
    }
}

fn input_positions() -> Vec<i64> {
    (-10_000..=10_000).collect()
}

fn sequential() {
    let positions = input_positions();
    let sum_initial_positions = positions.iter().sum::<i64>();
    println!("sum_initial_positions = {sum_initial_positions}");

    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let final_positions: Vec<_> = positions
        .iter()
        .copied()
        .map(|position| random_walk(&mut rng, position, 100))
        .collect();
    let sum_final_positions = final_positions.iter().sum::<i64>();
    println!("sum_final_positions = {sum_final_positions}");
}

fn parallel() {
    let positions = input_positions();
    let sum_initial_positions = positions.iter().sum::<i64>();
    println!("sum_initial_positions = {sum_initial_positions}");

    let final_positions: Vec<_> = positions
        .par()
        .copied()
        .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64))
        .map(|rng, position| random_walk(rng, position, 100))
        .collect();
    let sum_final_positions = final_positions.iter().sum::<i64>();
    println!("sum_final_positions = {sum_final_positions}");
}

fn main() {
    println!("\n\nSEQUENTIAL");
    sequential();

    println!("\n\nPARALLEL");
    parallel();
}
