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
    (-50..=50).collect()
}

#[test]
fn using_rng_map() {
    let positions = input_positions();
    let _ = positions.iter().sum::<i64>();

    let final_positions: Vec<_> = positions
        .par()
        .copied()
        .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64))
        .map(|rng, position| random_walk(rng, position, 100))
        .collect();
    let _ = final_positions.iter().sum::<i64>();
}

#[test]
fn using_rng_xap() {
    let positions = input_positions();
    let _ = positions.iter().sum::<i64>();

    let final_positions: Vec<_> = positions
        .par()
        .copied()
        .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64))
        .filter(|rng, _| rng.random_bool(0.7))
        .map(|rng, position| random_walk(rng, position, 100))
        .collect();
    let _ = final_positions.iter().sum::<i64>();
}

#[cfg(not(miri))]
#[test]
fn using_rng_xap_long() {
    let positions = input_positions();
    let _ = positions.iter().sum::<i64>();

    let final_positions: Vec<_> = positions
        .par()
        .copied()
        .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64))
        .filter(|rng, _| rng.random_bool(0.7))
        .filter_map(|rng, position| rng.random_bool(0.9).then_some(position))
        .filter(|rng, _| rng.random_bool(0.7))
        .flat_map(|_, position| [position, position])
        .map(|rng, position| random_walk(rng, position, 100))
        .collect();
    let _ = final_positions.iter().sum::<i64>();
}
