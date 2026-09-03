use crate::collectables::par_extend::ParExtend;
use alloc::{vec, vec::Vec};
use std::collections::HashSet;

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut set: HashSet<i32> = HashSet::new();
    let results: Vec<HashSet<i32>> = Vec::new();

    set.extend_merge_ordered_infallibles(results);
    assert!(set.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut set: HashSet<i32> = HashSet::from([1, 2, 3]);
    let t0 = HashSet::<i32>::default();
    let t1 = HashSet::<i32>::default();

    set.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 3]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut set = HashSet::new();
    let mut t0 = HashSet::default();

    HashSet::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);

    set.extend_merge_ordered_infallibles(vec![t0]);
    let expected: HashSet<i32> = HashSet::from([10, 20, 30]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut set = HashSet::new();
    let mut t0 = HashSet::default();

    HashSet::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    HashSet::add_ordered_thread_value(&mut t0, 1, 3);
    HashSet::add_ordered_thread_values(&mut t0, 2, vec![4, 5, 6]);

    set.extend_merge_ordered_infallibles(vec![t0]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 3, 4, 5, 6]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads() {
    let mut set = HashSet::new();
    let mut t0 = HashSet::default();
    let mut t1 = HashSet::default();

    HashSet::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    HashSet::add_ordered_thread_values(&mut t0, 2, vec![5, 6]);

    HashSet::add_ordered_thread_values(&mut t1, 1, vec![3, 4]);
    HashSet::add_ordered_thread_values(&mut t1, 3, vec![7, 8]);

    set.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_set() {
    let mut set = HashSet::from([100, 200]);
    let mut t0 = HashSet::default();
    let mut t1 = HashSet::default();

    HashSet::add_ordered_thread_value(&mut t0, 0, 1);
    HashSet::add_ordered_thread_value(&mut t1, 1, 2);

    set.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 100, 200]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_values() {
    let mut set = HashSet::from([10]);
    let mut t0 = HashSet::default();
    let mut t1 = HashSet::default();

    HashSet::add_ordered_thread_values(&mut t0, 0, vec![10, 20]);
    HashSet::add_ordered_thread_values(&mut t1, 1, vec![20, 30]);

    set.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([10, 20, 30]);
    assert_eq!(set, expected);
}

// extend_from_thread_results tests

#[test]
fn extend_from_thread_results_empty() {
    let mut set: HashSet<i32> = HashSet::new();
    let results: Vec<HashSet<i32>> = Vec::new();

    set.extend_merge_infallibles(results);
    assert!(set.is_empty());
}

#[test]
fn extend_from_thread_results_empty_threads() {
    let mut set: HashSet<i32> = HashSet::from([1, 2, 3]);
    let t0 = HashSet::<i32>::default();
    let t1 = HashSet::<i32>::default();

    set.extend_merge_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 3]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_thread_results_single_thread() {
    let mut set = HashSet::new();
    let mut t0 = HashSet::default();

    HashSet::add_thread_value(&mut t0, 10);
    HashSet::add_thread_values(&mut t0, vec![20, 30]);

    set.extend_merge_infallibles(vec![t0]);
    let expected: HashSet<i32> = HashSet::from([10, 20, 30]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_thread_results_multiple_threads() {
    let mut set = HashSet::new();
    let mut t0 = HashSet::default();
    let mut t1 = HashSet::default();

    HashSet::add_thread_value(&mut t0, 1);
    HashSet::add_thread_values(&mut t0, vec![2, 3]);

    HashSet::add_thread_value(&mut t1, 4);
    HashSet::add_thread_values(&mut t1, vec![5, 6]);

    set.extend_merge_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 3, 4, 5, 6]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_thread_results_append_to_non_empty_set() {
    let mut set = HashSet::from([100, 200]);
    let mut t0 = HashSet::default();
    let mut t1 = HashSet::default();

    HashSet::add_thread_value(&mut t0, 1);
    HashSet::add_thread_value(&mut t1, 2);

    set.extend_merge_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([1, 2, 100, 200]);
    assert_eq!(set, expected);
}

#[test]
fn extend_from_thread_results_duplicate_values() {
    let mut set = HashSet::from([10]);
    let mut t0 = HashSet::default();
    let mut t1 = HashSet::default();

    HashSet::add_thread_values(&mut t0, vec![10, 20]);
    HashSet::add_thread_values(&mut t1, vec![20, 30]);

    set.extend_merge_infallibles(vec![t0, t1]);
    let expected: HashSet<i32> = HashSet::from([10, 20, 30]);
    assert_eq!(set, expected);
}
