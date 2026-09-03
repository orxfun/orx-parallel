use crate::collectables::par_extend::ParExtend;
use crate::{IntoParIter, IterationOrder, Par};
use alloc::collections::BinaryHeap;
use alloc::{vec, vec::Vec};

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut heap: BinaryHeap<i32> = BinaryHeap::new();
    let results: Vec<BinaryHeap<i32>> = Vec::new();

    heap.extend_merge_ordered_infallibles(results);
    assert!(heap.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut heap: BinaryHeap<i32> = BinaryHeap::from([1, 2, 3]);
    let t0 = BinaryHeap::<i32>::default();
    let t1 = BinaryHeap::<i32>::default();

    heap.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected = vec![1, 2, 3];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut heap = BinaryHeap::new();
    let mut t0 = BinaryHeap::default();

    BinaryHeap::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);

    heap.extend_merge_ordered_infallibles(vec![t0]);
    let expected = vec![10, 20, 30];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut heap = BinaryHeap::new();
    let mut t0 = BinaryHeap::default();

    BinaryHeap::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    BinaryHeap::add_ordered_thread_value(&mut t0, 1, 3);
    BinaryHeap::add_ordered_thread_values(&mut t0, 2, vec![4, 5, 6]);

    heap.extend_merge_ordered_infallibles(vec![t0]);
    let expected = vec![1, 2, 3, 4, 5, 6];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads() {
    let mut heap = BinaryHeap::new();
    let mut t0 = BinaryHeap::default();
    let mut t1 = BinaryHeap::default();

    BinaryHeap::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    BinaryHeap::add_ordered_thread_values(&mut t0, 2, vec![5, 6]);

    BinaryHeap::add_ordered_thread_values(&mut t1, 1, vec![3, 4]);
    BinaryHeap::add_ordered_thread_values(&mut t1, 3, vec![7, 8]);

    heap.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected = vec![1, 2, 3, 4, 5, 6, 7, 8];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_heap() {
    let mut heap = BinaryHeap::from([100, 200]);
    let mut t0 = BinaryHeap::default();
    let mut t1 = BinaryHeap::default();

    BinaryHeap::add_ordered_thread_value(&mut t0, 0, 1);
    BinaryHeap::add_ordered_thread_value(&mut t1, 1, 2);

    heap.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected = vec![1, 2, 100, 200];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_values() {
    let mut heap = BinaryHeap::from([10]);
    let mut t0 = BinaryHeap::default();
    let mut t1 = BinaryHeap::default();

    BinaryHeap::add_ordered_thread_values(&mut t0, 0, vec![10, 20]);
    BinaryHeap::add_ordered_thread_values(&mut t1, 1, vec![20, 30]);

    heap.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected = vec![10, 10, 20, 20, 30];
    assert_eq!(heap.into_sorted_vec(), expected);
}

// extend_from_thread_results tests

#[test]
fn extend_from_thread_results_empty() {
    let mut heap: BinaryHeap<i32> = BinaryHeap::new();
    let results: Vec<BinaryHeap<i32>> = Vec::new();

    heap.extend_merge_infallibles(results);
    assert!(heap.is_empty());
}

#[test]
fn extend_from_thread_results_empty_threads() {
    let mut heap: BinaryHeap<i32> = BinaryHeap::from([1, 2, 3]);
    let t0 = BinaryHeap::<i32>::default();
    let t1 = BinaryHeap::<i32>::default();

    heap.extend_merge_infallibles(vec![t0, t1]);
    let expected = vec![1, 2, 3];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_thread_results_single_thread() {
    let mut heap = BinaryHeap::new();
    let mut t0 = BinaryHeap::default();

    BinaryHeap::add_thread_value(&mut t0, 10);
    BinaryHeap::add_thread_values(&mut t0, vec![20, 30]);

    heap.extend_merge_infallibles(vec![t0]);
    let expected = vec![10, 20, 30];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_thread_results_multiple_threads() {
    let mut heap = BinaryHeap::new();
    let mut t0 = BinaryHeap::default();
    let mut t1 = BinaryHeap::default();

    BinaryHeap::add_thread_value(&mut t0, 1);
    BinaryHeap::add_thread_values(&mut t0, vec![2, 3]);

    BinaryHeap::add_thread_value(&mut t1, 4);
    BinaryHeap::add_thread_values(&mut t1, vec![5, 6]);

    heap.extend_merge_infallibles(vec![t0, t1]);
    let expected = vec![1, 2, 3, 4, 5, 6];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_thread_results_append_to_non_empty_heap() {
    let mut heap = BinaryHeap::from([100, 200]);
    let mut t0 = BinaryHeap::default();
    let mut t1 = BinaryHeap::default();

    BinaryHeap::add_thread_value(&mut t0, 1);
    BinaryHeap::add_thread_value(&mut t1, 2);

    heap.extend_merge_infallibles(vec![t0, t1]);
    let expected = vec![1, 2, 100, 200];
    assert_eq!(heap.into_sorted_vec(), expected);
}

#[test]
fn extend_from_thread_results_duplicate_values() {
    let mut heap = BinaryHeap::from([10]);
    let mut t0 = BinaryHeap::default();
    let mut t1 = BinaryHeap::default();

    BinaryHeap::add_thread_values(&mut t0, vec![10, 20]);
    BinaryHeap::add_thread_values(&mut t1, vec![20, 30]);

    heap.extend_merge_infallibles(vec![t0, t1]);
    let expected = vec![10, 10, 20, 20, 30];
    assert_eq!(heap.into_sorted_vec(), expected);
}

// parallel iterator collect tests

#[test]
fn par_collect_ordered() {
    let input: Vec<i32> = (0..100).collect();
    let collected: BinaryHeap<i32> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Ordered)
        .map(|x| x * 2)
        .collect();
    let collected_vec = collected.into_sorted_vec();
    let expected: Vec<i32> = input.into_iter().map(|x| x * 2).collect();
    assert_eq!(collected_vec, expected);
}

#[test]
fn par_collect_into_ordered() {
    let input: Vec<i32> = (0..100).collect();
    let mut dst = BinaryHeap::from([-2, -1]);
    input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Ordered)
        .map(|x| x * 2)
        .collect_into(&mut dst);
    let dst_vec = dst.into_sorted_vec();
    let mut expected: Vec<i32> = vec![-2, -1];
    expected.extend(input.into_iter().map(|x| x * 2));
    expected.sort();
    assert_eq!(dst_vec, expected);
}

#[test]
fn par_collect_arbitrary() {
    let input: Vec<i32> = (0..100).collect();
    let collected: BinaryHeap<i32> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map(|x| x * 2)
        .collect();
    let collected_vec = collected.into_sorted_vec();
    let expected: Vec<i32> = input.into_iter().map(|x| x * 2).collect();
    assert_eq!(collected_vec, expected);
}

#[test]
fn par_collect_into_arbitrary() {
    let input: Vec<i32> = (0..100).collect();
    let mut dst = BinaryHeap::from([-2, -1]);
    input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map(|x| x * 2)
        .collect_into(&mut dst);
    let dst_vec = dst.into_sorted_vec();
    let mut expected: Vec<i32> = vec![-2, -1];
    expected.extend(input.into_iter().map(|x| x * 2));
    expected.sort();
    assert_eq!(dst_vec, expected);
}
