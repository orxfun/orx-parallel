use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[test]
fn use_fun() {
    let input = 0..10000;
    let values: Vec<_> = input
        .par()
        .map(|x| 2 * x)
        // a thread-local RNG is consumed while processing,
        // and we only keep the transformed values.
        .using(Use::fun(|thread_idx| {
            ChaCha8Rng::seed_from_u64(1000 + thread_idx as u64)
        }))
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
fn use_fun2() {
    let input = 0..10000;
    let values: Vec<_> = input
        .par()
        .map(|x| 2 * x)
        // a thread-local RNG is consumed while processing,
        // and we only keep the transformed values.
        .using(Use::fun(|thread_idx| {
            ChaCha8Rng::seed_from_u64(1000 + thread_idx as u64)
        }))
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
    let par = input.par().map(|x| 2 * x);
    let mut use_vec = Use::vec(|_| 0);
    let use_par = par.using(&mut use_vec);
    use_par.for_each(|thread_sum, x| *thread_sum += x);

    let thread_sums = use_vec.into_vec();

    let grand_total: usize = thread_sums.into_iter().sum();
    assert_eq!(grand_total, (input.len() - 1) * input.len());
}

#[test]
fn use_slice() {
    let num_threads = 8;

    let mut thread_sums = vec![0; num_threads];

    let input = 0..10000;
    let par = input.par().map(|x| 2 * x);
    // TODO: this shouldn't be necessary
    let par = par.num_threads(num_threads);
    let use_par = par.using(Use::slice(&mut thread_sums));
    use_par.for_each(|thread_sum, x| *thread_sum += x);

    let grand_total: usize = thread_sums.into_iter().sum();
    assert_eq!(grand_total, (input.len() - 1) * input.len());
}
