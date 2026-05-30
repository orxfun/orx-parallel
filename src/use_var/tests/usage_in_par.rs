use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test]
fn use_fun() {
    let input = 0..10000;
    let values: Vec<_> = input
        .par()
        .map(|x| 2 * x)
        // a thread-local RNG is consumed while processing,
        // and we only keep the transformed values.
        .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(1000 + thread_idx as u64))
        .map(|rng, x| x + rng.random_range(0..10))
        .collect();

    assert_eq!(values.len(), input.len());
    assert!(
        values
            .iter()
            .zip(input)
            .all(|(value, x)| *value >= 2 * x && *value < 2 * x + 10)
    );
}

#[test]
fn use_vec() {
    let input = 0..10000;
    let mut use_vec = UseVec::new(|_| 0);
    input
        .par()
        .map(|x| 2 * x)
        .use_vec(&mut use_vec)
        .for_each(|thread_sum, x| *thread_sum += x);

    let thread_sums = use_vec.into_vec();

    let grand_total: usize = thread_sums.into_iter().sum();
    assert_eq!(grand_total, (input.len() - 1) * input.len());
}

#[test_matrix([1, 4, 16, 100])]
fn use_slice(slice_len: usize) {
    let mut thread_sums = vec![0; slice_len];

    let input = 0..10000;
    input
        .par()
        .map(|x| 2 * x)
        .use_slice(&mut thread_sums)
        .for_each(|thread_sum, x| *thread_sum += x);

    let grand_total: usize = thread_sums.into_iter().sum();
    assert_eq!(grand_total, (input.len() - 1) * input.len());
}

#[test]
#[should_panic]
fn use_slice_panics_when_empty() {
    let mut thread_sums = vec![0; 0];

    let input = 0..10000;
    input
        .par()
        .map(|x| 2 * x)
        .use_slice(&mut thread_sums)
        .for_each(|thread_sum, x| *thread_sum += x);

    let grand_total: usize = thread_sums.into_iter().sum();
    assert_eq!(grand_total, (input.len() - 1) * input.len());
}
