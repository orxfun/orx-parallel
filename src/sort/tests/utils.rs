use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub fn create_input<T: Clone>(
    len: usize,
    sorted_value_at: impl Fn(usize) -> T,
    number_of_swaps: usize,
) -> Vec<T> {
    let mut input: Vec<_> = (0..len).map(sorted_value_at).collect();
    if len > 0 {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..number_of_swaps {
            let i = rng.random_range(0..len);
            let j = rng.random_range(0..len);
            input.swap(i, j);
        }
    }
    input
}

pub fn create_input_and_sorted<T: Clone + Ord>(
    len: usize,
    sorted_value_at: impl Fn(usize) -> T,
    number_of_swaps: usize,
) -> (Vec<T>, Vec<T>) {
    let input = create_input(len, sorted_value_at, number_of_swaps);
    let mut sorted = input.clone();
    sorted.sort();
    (input, sorted)
}
