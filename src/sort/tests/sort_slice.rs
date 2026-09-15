use crate::Params;
use crate::{runner::default_runner, sort::sort_slice::par_experimental_sort};
use alloc::string::String;
use alloc::vec::Vec;

#[test]
fn test_sort_empty_and_single() {
    let mut runner = default_runner();
    let mut empty: [i32; 0] = [];
    par_experimental_sort(&mut empty, &mut runner, Params::default());

    let mut single = [42];
    par_experimental_sort(&mut single, &mut runner, Params::default());
    assert_eq!(single, [42]);
}

#[test]
fn test_sort_small_slices() {
    let mut runner = default_runner();
    let mut data = [9, 3, 7, 1, 5, 2, 8, 4, 6];
    par_experimental_sort(&mut data, &mut runner, Params::default());
    assert_eq!(data, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_sort_medium_and_large_random() {
    let mut runner = default_runner();

    for size in [500, 1024, 2048, 5000, 20000, 50000] {
        let mut data: Vec<i32> = (0..size as u64)
            .map(|i| (i.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFFFFFF) as i32)
            .collect();
        let mut expected = data.clone();
        expected.sort_unstable();

        par_experimental_sort(&mut data, &mut runner, Params::default());
        assert_eq!(data, expected, "Failed for size {}", size);
    }
}

#[test]
fn test_sort_sorted_and_reversed() {
    let mut runner = default_runner();
    let size = 10000;

    let mut sorted: Vec<i32> = (0..size).collect();
    par_experimental_sort(&mut sorted, &mut runner, Params::default());
    assert!(sorted.windows(2).all(|w| w[0] <= w[1]));

    let mut reversed: Vec<i32> = (0..size).rev().collect();
    par_experimental_sort(&mut reversed, &mut runner, Params::default());
    assert!(reversed.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn test_sort_high_duplicates() {
    let mut runner = default_runner();
    let size = 20000;
    let mut data: Vec<i32> = (0..size).map(|i| i % 7).collect();
    let mut expected = data.clone();
    expected.sort_unstable();

    par_experimental_sort(&mut data, &mut runner, Params::default());
    assert_eq!(data, expected);
}

#[test]
fn test_sort_non_copy_types() {
    let mut runner = default_runner();
    let size = 5000;
    let mut data: Vec<String> = (0..size)
        .map(|i| alloc::format!("item_{:06}", (i * 7919) % size))
        .collect();
    let mut expected = data.clone();
    expected.sort();

    par_experimental_sort(&mut data, &mut runner, Params::default());
    assert_eq!(data, expected);
}

#[test]
fn test_sort_num_threads_configs() {
    for nt in [1, 2, 4, 8] {
        let mut runner = default_runner();
        let size = 10000;
        let mut data: Vec<i32> = (0..size as u64)
            .map(|i| (i.wrapping_mul(2654435761) & 0x7FFFFFFF) as i32)
            .collect();
        let mut expected = data.clone();
        expected.sort_unstable();

        let params = Params::default().with_num_threads(nt);
        par_experimental_sort(&mut data, &mut runner, params);
        assert_eq!(data, expected, "Failed for num_threads = {}", nt);
    }
}
