use crate::collectables::par_extend::ParExtend;
use alloc::{vec, vec::Vec};
use std::collections::HashMap;

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut map: HashMap<i32, &'static str> = HashMap::new();
    let results: Vec<HashMap<i32, &'static str>> = Vec::new();

    map.extend_from_ordered_thread_results(results);
    assert!(map.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut map: HashMap<i32, &'static str> = HashMap::from([(1, "a"), (2, "b"), (3, "c")]);
    let t0 = HashMap::<i32, &'static str>::default();
    let t1 = HashMap::<i32, &'static str>::default();

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected = HashMap::from([(1, "a"), (2, "b"), (3, "c")]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut map = HashMap::new();
    let mut t0 = HashMap::default();

    HashMap::add_ordered_thread_values(
        &mut t0,
        0,
        vec![(10, "ten"), (20, "twenty"), (30, "thirty")],
    );

    map.extend_from_ordered_thread_results(vec![t0]);
    let expected = HashMap::from([(10, "ten"), (20, "twenty"), (30, "thirty")]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut map = HashMap::new();
    let mut t0 = HashMap::default();

    HashMap::add_ordered_thread_values(&mut t0, 0, vec![(1, "one"), (2, "two")]);
    HashMap::add_ordered_thread_value(&mut t0, 1, (3, "three"));
    HashMap::add_ordered_thread_values(&mut t0, 2, vec![(4, "four"), (5, "five"), (6, "six")]);

    map.extend_from_ordered_thread_results(vec![t0]);
    let expected = HashMap::from([
        (1, "one"),
        (2, "two"),
        (3, "three"),
        (4, "four"),
        (5, "five"),
        (6, "six"),
    ]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads() {
    let mut map = HashMap::new();
    let mut t0 = HashMap::default();
    let mut t1 = HashMap::default();

    HashMap::add_ordered_thread_values(&mut t0, 0, vec![(1, "one"), (2, "two")]);
    HashMap::add_ordered_thread_values(&mut t0, 2, vec![(5, "five"), (6, "six")]);

    HashMap::add_ordered_thread_values(&mut t1, 1, vec![(3, "three"), (4, "four")]);
    HashMap::add_ordered_thread_values(&mut t1, 3, vec![(7, "seven"), (8, "eight")]);

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected = HashMap::from([
        (1, "one"),
        (2, "two"),
        (3, "three"),
        (4, "four"),
        (5, "five"),
        (6, "six"),
        (7, "seven"),
        (8, "eight"),
    ]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_map() {
    let mut map = HashMap::from([(100, "hundred"), (200, "two_hundred")]);
    let mut t0 = HashMap::default();
    let mut t1 = HashMap::default();

    HashMap::add_ordered_thread_value(&mut t0, 0, (1, "one"));
    HashMap::add_ordered_thread_value(&mut t1, 1, (2, "two"));

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected = HashMap::from([
        (1, "one"),
        (2, "two"),
        (100, "hundred"),
        (200, "two_hundred"),
    ]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_keys() {
    let mut map = HashMap::from([(10, "initial_10")]);
    let mut t0 = HashMap::default();
    let mut t1 = HashMap::default();

    HashMap::add_ordered_thread_values(&mut t0, 0, vec![(10, "t0_10"), (20, "t0_20")]);
    HashMap::add_ordered_thread_values(&mut t1, 1, vec![(20, "t1_20"), (30, "t1_30")]);

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    assert!(map.contains_key(&10));
    assert!(map.contains_key(&20));
    assert_eq!(map.get(&30), Some(&"t1_30"));
}
