use crate::collectables::par_extend::ParExtend;
use crate::{IntoParIter, IterationOrder, Par};
use alloc::collections::LinkedList;
use alloc::{vec, vec::Vec};

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut list: LinkedList<i32> = LinkedList::new();
    let results: Vec<LinkedList<i32>> = Vec::new();

    list.extend_merge_ordered_infallibles(results);
    assert!(list.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut list: LinkedList<i32> = LinkedList::from([1, 2, 3]);
    let t0 = LinkedList::<i32>::default();
    let t1 = LinkedList::<i32>::default();

    list.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: LinkedList<i32> = LinkedList::from([1, 2, 3]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut list = LinkedList::new();
    let mut t0 = LinkedList::default();

    LinkedList::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);

    list.extend_merge_ordered_infallibles(vec![t0]);
    let expected: LinkedList<i32> = LinkedList::from([10, 20, 30]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut list = LinkedList::new();
    let mut t0 = LinkedList::default();

    LinkedList::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    LinkedList::add_ordered_thread_value(&mut t0, 1, 3);
    LinkedList::add_ordered_thread_values(&mut t0, 2, vec![4, 5, 6]);

    list.extend_merge_ordered_infallibles(vec![t0]);
    let expected: LinkedList<i32> = LinkedList::from([1, 2, 3, 4, 5, 6]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads() {
    let mut list = LinkedList::new();
    let mut t0 = LinkedList::default();
    let mut t1 = LinkedList::default();

    LinkedList::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    LinkedList::add_ordered_thread_values(&mut t0, 2, vec![5, 6]);

    LinkedList::add_ordered_thread_values(&mut t1, 1, vec![3, 4]);
    LinkedList::add_ordered_thread_values(&mut t1, 3, vec![7, 8]);

    list.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: LinkedList<i32> = LinkedList::from([1, 2, 5, 6, 3, 4, 7, 8]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_list() {
    let mut list = LinkedList::from([100, 200]);
    let mut t0 = LinkedList::default();
    let mut t1 = LinkedList::default();

    LinkedList::add_ordered_thread_value(&mut t0, 0, 1);
    LinkedList::add_ordered_thread_value(&mut t1, 1, 2);

    list.extend_merge_ordered_infallibles(vec![t0, t1]);
    let expected: LinkedList<i32> = LinkedList::from([100, 200, 1, 2]);
    assert_eq!(list, expected);
}

// extend_from_thread_results tests

#[test]
fn extend_from_thread_results_empty() {
    let mut list: LinkedList<i32> = LinkedList::new();
    let results: Vec<LinkedList<i32>> = Vec::new();

    list.extend_merge_infallibles(results);
    assert!(list.is_empty());
}

#[test]
fn extend_from_thread_results_empty_threads() {
    let mut list: LinkedList<i32> = LinkedList::from([1, 2, 3]);
    let t0 = LinkedList::<i32>::default();
    let t1 = LinkedList::<i32>::default();

    list.extend_merge_infallibles(vec![t0, t1]);
    let expected: LinkedList<i32> = LinkedList::from([1, 2, 3]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_thread_results_single_thread() {
    let mut list = LinkedList::new();
    let mut t0 = LinkedList::default();

    LinkedList::add_thread_value(&mut t0, 10);
    LinkedList::add_thread_values(&mut t0, vec![20, 30]);

    list.extend_merge_infallibles(vec![t0]);
    let expected: LinkedList<i32> = LinkedList::from([10, 20, 30]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_thread_results_multiple_threads() {
    let mut list = LinkedList::new();
    let mut t0 = LinkedList::default();
    let mut t1 = LinkedList::default();

    LinkedList::add_thread_value(&mut t0, 1);
    LinkedList::add_thread_values(&mut t0, vec![2, 3]);

    LinkedList::add_thread_value(&mut t1, 4);
    LinkedList::add_thread_values(&mut t1, vec![5, 6]);

    list.extend_merge_infallibles(vec![t0, t1]);
    let expected: LinkedList<i32> = LinkedList::from([1, 2, 3, 4, 5, 6]);
    assert_eq!(list, expected);
}

#[test]
fn extend_from_thread_results_append_to_non_empty_list() {
    let mut list = LinkedList::from([100, 200]);
    let mut t0 = LinkedList::default();
    let mut t1 = LinkedList::default();

    LinkedList::add_thread_value(&mut t0, 1);
    LinkedList::add_thread_value(&mut t1, 2);

    list.extend_merge_infallibles(vec![t0, t1]);
    let expected: LinkedList<i32> = LinkedList::from([100, 200, 1, 2]);
    assert_eq!(list, expected);
}

// parallel iterator collect tests

#[test]
fn par_collect_ordered() {
    let input: Vec<i32> = (0..100).collect();
    let collected: LinkedList<i32> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Ordered)
        .map(|x| x * 2)
        .collect();
    let mut collected_vec: Vec<i32> = collected.into_iter().collect();
    let mut expected: Vec<i32> = input.into_iter().map(|x| x * 2).collect();
    collected_vec.sort();
    expected.sort();
    assert_eq!(collected_vec, expected);
}

#[test]
fn par_collect_into_ordered() {
    let input: Vec<i32> = (0..100).collect();
    let mut dst = LinkedList::from([-2, -1]);
    input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Ordered)
        .map(|x| x * 2)
        .collect_into(&mut dst);
    let mut dst_vec: Vec<i32> = dst.into_iter().collect();
    let mut expected: Vec<i32> = vec![-2, -1];
    expected.extend(input.into_iter().map(|x| x * 2));
    dst_vec.sort();
    expected.sort();
    assert_eq!(dst_vec, expected);
}

#[test]
fn par_collect_arbitrary() {
    let input: Vec<i32> = (0..100).collect();
    let collected: LinkedList<i32> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map(|x| x * 2)
        .collect();
    let mut collected_vec: Vec<i32> = collected.into_iter().collect();
    let mut expected: Vec<i32> = input.into_iter().map(|x| x * 2).collect();
    collected_vec.sort();
    expected.sort();
    assert_eq!(collected_vec, expected);
}

#[test]
fn par_collect_into_arbitrary() {
    let input: Vec<i32> = (0..100).collect();
    let mut dst = LinkedList::from([-2, -1]);
    input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map(|x| x * 2)
        .collect_into(&mut dst);
    let mut dst_vec: Vec<i32> = dst.into_iter().collect();
    let mut expected: Vec<i32> = vec![-2, -1];
    expected.extend(input.into_iter().map(|x| x * 2));
    dst_vec.sort();
    expected.sort();
    assert_eq!(dst_vec, expected);
}
