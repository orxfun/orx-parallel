use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub fn create_input<T>(
    len: usize,
    sorted_value_at: impl Fn(usize) -> T,
    number_of_swaps: usize,
) -> Vec<T> {
    let mut v: Vec<_> = (0..len).map(sorted_value_at).collect();
    if len > 0 {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..number_of_swaps {
            let i = rng.random_range(0..len);
            let j = rng.random_range(0..len);
            v.swap(i, j);
        }
    }
    v
}
