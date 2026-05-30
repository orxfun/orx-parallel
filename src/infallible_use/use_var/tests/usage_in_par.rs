use crate::*;
use alloc::vec;
use std::dbg;

#[test]
fn slice_xyz() {
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
