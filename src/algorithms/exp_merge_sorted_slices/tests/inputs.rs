use alloc::string::{String, ToString};
use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy)]
pub enum SortKind {
    Sorted,
    ReverseSorted,
    Mixed,
}

pub fn sorted_slices(
    left_len: usize,
    total_len: usize,
    sort: SortKind,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut all: Vec<_> = (0..total_len).map(|x| x.to_string()).collect();
    match sort {
        SortKind::Sorted => all.sort(),
        SortKind::ReverseSorted => {
            all.sort();
            all.reverse();
        }
        SortKind::Mixed => {
            let num_shuffles = 10 * total_len;
            let mut rng = ChaCha8Rng::seed_from_u64(42);
            for _ in 0..num_shuffles {
                let i = rng.random_range(0..total_len);
                let j = rng.random_range(0..total_len);
                all.swap(i, j);
            }
        }
    }

    let (left, right) = all.split_at(left_len);
    let mut left: Vec<_> = left.iter().cloned().collect();
    let mut right: Vec<_> = right.iter().cloned().collect();
    left.sort();
    right.sort();
    (all, left, right)
}
