use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::btree_set::SetAndPositions;
use alloc::collections::BTreeSet;
use alloc::{vec, vec::Vec};

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut set: BTreeSet<i32> = BTreeSet::new();
    let results: Vec<SetAndPositions<i32>> = Vec::new();

    set.extend_from_ordered_thread_results(results);
    assert!(set.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut set: BTreeSet<i32> = BTreeSet::from([1, 2, 3]);
    let t0 = SetAndPositions::<i32>::default();
    let t1 = SetAndPositions::<i32>::default();

    set.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected: BTreeSet<i32> = BTreeSet::from([1, 2, 3]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();

    BTreeSet::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);

    set.extend_from_ordered_thread_results(vec![t0]);
    let expected: BTreeSet<i32> = BTreeSet::from([10, 20, 30]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();

    BTreeSet::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    BTreeSet::add_ordered_thread_value(&mut t0, 1, 3);
    BTreeSet::add_ordered_thread_values(&mut t0, 2, vec![4, 5, 6]);

    set.extend_from_ordered_thread_results(vec![t0]);
    let expected: BTreeSet<i32> = BTreeSet::from([1, 2, 3, 4, 5, 6]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads_in_order() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();
    let mut t1 = SetAndPositions::default();

    BTreeSet::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    BTreeSet::add_ordered_thread_values(&mut t0, 2, vec![5, 6]);

    BTreeSet::add_ordered_thread_values(&mut t1, 1, vec![3, 4]);
    BTreeSet::add_ordered_thread_values(&mut t1, 3, vec![7, 8]);

    set.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected: BTreeSet<i32> = BTreeSet::from([1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_interleaved_threads() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();
    let mut t1 = SetAndPositions::default();
    let mut t2 = SetAndPositions::default();

    // t0 has chunks 3 and 5
    BTreeSet::add_ordered_thread_values(&mut t0, 3, vec![7, 8]);
    BTreeSet::add_ordered_thread_value(&mut t0, 5, 11);

    // t1 has chunks 0 and 2
    BTreeSet::add_ordered_thread_values(&mut t1, 0, vec![1, 2, 3]);
    BTreeSet::add_ordered_thread_value(&mut t1, 2, 6);

    // t2 has chunks 1 and 4
    BTreeSet::add_ordered_thread_values(&mut t2, 1, vec![4, 5]);
    BTreeSet::add_ordered_thread_values(&mut t2, 4, vec![9, 10]);

    set.extend_from_ordered_thread_results(vec![t0, t1, t2]);
    let expected: BTreeSet<i32> = BTreeSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_set() {
    let mut set = BTreeSet::from([100, 200]);
    let mut t0 = SetAndPositions::default();
    let mut t1 = SetAndPositions::default();

    BTreeSet::add_ordered_thread_value(&mut t0, 0, 1);
    BTreeSet::add_ordered_thread_value(&mut t1, 1, 2);

    set.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected: BTreeSet<i32> = BTreeSet::from([1, 2, 100, 200]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_empty_iterators_ignored() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();

    // Empty iterator added should not create a chunk
    BTreeSet::add_ordered_thread_values(&mut t0, 0, Vec::<i32>::new());
    BTreeSet::add_ordered_thread_values(&mut t0, 1, vec![10, 20]);

    set.extend_from_ordered_thread_results(vec![t0]);
    let expected: BTreeSet<i32> = BTreeSet::from([10, 20]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_many_threads_and_chunks() {
    let mut set = BTreeSet::new();
    let num_threads = 8;
    let chunks_per_thread = 10;
    let mut thread_results: Vec<SetAndPositions<i32>> = (0..num_threads)
        .map(|_| SetAndPositions::default())
        .collect();

    for chunk_idx in 0..(num_threads * chunks_per_thread) {
        let t = chunk_idx % num_threads;
        let val = chunk_idx as i32 * 10;
        BTreeSet::add_ordered_thread_value(&mut thread_results[t], chunk_idx, val);
    }

    set.extend_from_ordered_thread_results(thread_results);
    let expected: BTreeSet<i32> = (0..(num_threads * chunks_per_thread))
        .map(|i| i as i32 * 10)
        .collect();
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_values_within_thread() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();

    // Add duplicate values within the same chunk and across chunks of the same thread
    BTreeSet::add_ordered_thread_values(&mut t0, 0, vec![5, 5, 10, 5]);
    BTreeSet::add_ordered_thread_value(&mut t0, 1, 10);
    BTreeSet::add_ordered_thread_values(&mut t0, 2, vec![15, 5, 20]);

    set.extend_from_ordered_thread_results(vec![t0]);
    let expected: BTreeSet<i32> = BTreeSet::from([5, 10, 15, 20]);
    assert_eq!(set, expected);
    let vec_collected: Vec<i32> = set.into_iter().collect();
    assert_eq!(vec_collected, vec![5, 10, 15, 20]);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_values_across_threads() {
    let mut set = BTreeSet::new();
    let mut t0 = SetAndPositions::default();
    let mut t1 = SetAndPositions::default();
    let mut t2 = SetAndPositions::default();

    // Chunk 0 on t1, Chunk 1 on t0, Chunk 2 on t2, Chunk 3 on t0
    BTreeSet::add_ordered_thread_values(&mut t1, 0, vec![10, 20, 30]);
    BTreeSet::add_ordered_thread_values(&mut t0, 1, vec![20, 30, 40]);
    BTreeSet::add_ordered_thread_values(&mut t2, 2, vec![10, 40, 50]);
    BTreeSet::add_ordered_thread_values(&mut t0, 3, vec![30, 50, 60]);

    set.extend_from_ordered_thread_results(vec![t0, t1, t2]);
    let expected: BTreeSet<i32> = BTreeSet::from([10, 20, 30, 40, 50, 60]);
    assert_eq!(set, expected);
    let vec_collected: Vec<i32> = set.into_iter().collect();
    assert_eq!(vec_collected, vec![10, 20, 30, 40, 50, 60]);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_values_with_existing_elements() {
    let mut set = BTreeSet::from([20, 40, 60]);
    let mut t0 = SetAndPositions::default();
    let mut t1 = SetAndPositions::default();

    BTreeSet::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);
    BTreeSet::add_ordered_thread_values(&mut t1, 1, vec![40, 50, 60]);

    set.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected: BTreeSet<i32> = BTreeSet::from([10, 20, 30, 40, 50, 60]);
    assert_eq!(set, expected);
}
