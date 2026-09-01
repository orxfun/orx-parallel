use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::col_and_pos::ColAndPos;
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};

type MapResults<K, V> = ColAndPos<BTreeMap<K, V>>;

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut map: BTreeMap<i32, &'static str> = BTreeMap::new();
    let results: Vec<MapResults<i32, &'static str>> = Vec::new();

    map.extend_from_ordered_thread_results(results);
    assert!(map.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut map: BTreeMap<i32, &'static str> = BTreeMap::from([(1, "a"), (2, "b"), (3, "c")]);
    let t0 = MapResults::<i32, &'static str>::default();
    let t1 = MapResults::<i32, &'static str>::default();

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected = BTreeMap::from([(1, "a"), (2, "b"), (3, "c")]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();

    BTreeMap::add_ordered_thread_values(
        &mut t0,
        0,
        vec![(10, "ten"), (20, "twenty"), (30, "thirty")],
    );

    map.extend_from_ordered_thread_results(vec![t0]);
    let expected = BTreeMap::from([(10, "ten"), (20, "twenty"), (30, "thirty")]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();

    BTreeMap::add_ordered_thread_values(&mut t0, 0, vec![(1, "one"), (2, "two")]);
    BTreeMap::add_ordered_thread_value(&mut t0, 1, (3, "three"));
    BTreeMap::add_ordered_thread_values(
        &mut t0,
        2,
        vec![(4, "four"), (5, "five"), (6, "six")],
    );

    map.extend_from_ordered_thread_results(vec![t0]);
    let expected = BTreeMap::from([
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
fn extend_from_ordered_thread_results_multiple_threads_in_order() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();
    let mut t1 = MapResults::default();

    BTreeMap::add_ordered_thread_values(&mut t0, 0, vec![(1, "one"), (2, "two")]);
    BTreeMap::add_ordered_thread_values(&mut t0, 2, vec![(5, "five"), (6, "six")]);

    BTreeMap::add_ordered_thread_values(&mut t1, 1, vec![(3, "three"), (4, "four")]);
    BTreeMap::add_ordered_thread_values(&mut t1, 3, vec![(7, "seven"), (8, "eight")]);

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected = BTreeMap::from([
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
fn extend_from_ordered_thread_results_interleaved_threads() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();
    let mut t1 = MapResults::default();
    let mut t2 = MapResults::default();

    // t0 has chunks 3 and 5
    BTreeMap::add_ordered_thread_values(&mut t0, 3, vec![(7, "seven"), (8, "eight")]);
    BTreeMap::add_ordered_thread_value(&mut t0, 5, (11, "eleven"));

    // t1 has chunks 0 and 2
    BTreeMap::add_ordered_thread_values(&mut t1, 0, vec![(1, "one"), (2, "two"), (3, "three")]);
    BTreeMap::add_ordered_thread_value(&mut t1, 2, (6, "six"));

    // t2 has chunks 1 and 4
    BTreeMap::add_ordered_thread_values(&mut t2, 1, vec![(4, "four"), (5, "five")]);
    BTreeMap::add_ordered_thread_values(&mut t2, 4, vec![(9, "nine"), (10, "ten")]);

    map.extend_from_ordered_thread_results(vec![t0, t1, t2]);
    let expected = BTreeMap::from([
        (1, "one"),
        (2, "two"),
        (3, "three"),
        (4, "four"),
        (5, "five"),
        (6, "six"),
        (7, "seven"),
        (8, "eight"),
        (9, "nine"),
        (10, "ten"),
        (11, "eleven"),
    ]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_map() {
    let mut map = BTreeMap::from([(100, "hundred"), (200, "two_hundred")]);
    let mut t0 = MapResults::default();
    let mut t1 = MapResults::default();

    BTreeMap::add_ordered_thread_value(&mut t0, 0, (1, "one"));
    BTreeMap::add_ordered_thread_value(&mut t1, 1, (2, "two"));

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    let expected = BTreeMap::from([
        (1, "one"),
        (2, "two"),
        (100, "hundred"),
        (200, "two_hundred"),
    ]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_empty_iterators_ignored() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();

    // Empty iterator added should not create a chunk
    BTreeMap::add_ordered_thread_values(&mut t0, 0, Vec::<(i32, &'static str)>::new());
    BTreeMap::add_ordered_thread_values(&mut t0, 1, vec![(10, "ten"), (20, "twenty")]);

    map.extend_from_ordered_thread_results(vec![t0]);
    let expected = BTreeMap::from([(10, "ten"), (20, "twenty")]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_many_threads_and_chunks() {
    let mut map = BTreeMap::new();
    let num_threads = 8;
    let chunks_per_thread = 10;
    let mut thread_results: Vec<MapResults<i32, i32>> = (0..num_threads)
        .map(|_| MapResults::default())
        .collect();

    for chunk_idx in 0..(num_threads * chunks_per_thread) {
        let t = chunk_idx % num_threads;
        let key = chunk_idx as i32 * 10;
        let val = chunk_idx as i32 * 100;
        BTreeMap::add_ordered_thread_value(&mut thread_results[t], chunk_idx, (key, val));
    }

    map.extend_from_ordered_thread_results(thread_results);
    let expected: BTreeMap<i32, i32> = (0..(num_threads * chunks_per_thread))
        .map(|i| (i as i32 * 10, i as i32 * 100))
        .collect();
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_keys_within_thread() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();

    // In add_ordered_thread_value, if a key is already in collected.values (which is a BTreeMap),
    // insert updates the value and returns Some(old_val), so inserted = false and position is not pushed.
    BTreeMap::add_ordered_thread_values(
        &mut t0,
        0,
        vec![(5, "v1"), (10, "v2")],
    );
    BTreeMap::add_ordered_thread_value(&mut t0, 1, (5, "v1_updated"));
    BTreeMap::add_ordered_thread_values(&mut t0, 2, vec![(15, "v3"), (20, "v4")]);

    map.extend_from_ordered_thread_results(vec![t0]);
    let expected = BTreeMap::from([
        (5, "v1_updated"),
        (10, "v2"),
        (15, "v3"),
        (20, "v4"),
    ]);
    assert_eq!(map, expected);
}

#[test]
fn extend_from_ordered_thread_results_duplicate_keys_across_threads() {
    let mut map = BTreeMap::new();
    let mut t0 = MapResults::default();
    let mut t1 = MapResults::default();
    let mut t2 = MapResults::default();

    // Chunk 0 on t1, Chunk 1 on t0, Chunk 2 on t2, Chunk 3 on t0
    // Chunk 0 (t1): (10, "v1_t1"), (20, "v2_t1")
    // Chunk 1 (t0): (20, "v2_t0_overwrite"), (30, "v3_t0")
    // Chunk 2 (t2): (10, "v1_t2_overwrite"), (40, "v4_t2")
    // Chunk 3 (t0): (30, "v3_t0_overwrite"), (50, "v5_t0")
    BTreeMap::add_ordered_thread_values(&mut t1, 0, vec![(10, "v1_t1"), (20, "v2_t1")]);
    BTreeMap::add_ordered_thread_values(&mut t0, 1, vec![(20, "v2_t0_overwrite"), (30, "v3_t0")]);
    BTreeMap::add_ordered_thread_values(&mut t2, 2, vec![(10, "v1_t2_overwrite"), (40, "v4_t2")]);
    BTreeMap::add_ordered_thread_values(&mut t0, 3, vec![(30, "v3_t0_overwrite"), (50, "v5_t0")]);

    map.extend_from_ordered_thread_results(vec![t0, t1, t2]);
    // Since chunks are processed in chunk index order (0, 1, 2, 3), later chunk values overwrite earlier ones.
    assert_eq!(map.get(&10), Some(&"v1_t2_overwrite"));
    assert_eq!(map.get(&20), Some(&"v2_t0_overwrite"));
    assert_eq!(map.get(&30), Some(&"v3_t0_overwrite"));
    assert_eq!(map.get(&40), Some(&"v4_t2"));
    assert_eq!(map.get(&50), Some(&"v5_t0"));
}

#[test]
fn extend_from_ordered_thread_results_duplicate_keys_with_existing_elements() {
    let mut map = BTreeMap::from([(20, "initial_20"), (40, "initial_40")]);
    let mut t0 = MapResults::default();
    let mut t1 = MapResults::default();

    BTreeMap::add_ordered_thread_values(&mut t0, 0, vec![(10, "new_10"), (20, "new_20")]);
    BTreeMap::add_ordered_thread_values(&mut t1, 1, vec![(40, "new_40"), (50, "new_50")]);

    map.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(map.get(&10), Some(&"new_10"));
    assert_eq!(map.get(&20), Some(&"new_20"));
    assert_eq!(map.get(&40), Some(&"new_40"));
    assert_eq!(map.get(&50), Some(&"new_50"));
}
