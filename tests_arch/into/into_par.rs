use crate::into::core::*;
use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_parallel::*;

const INPUT_LEN: usize = 1024;

#[test]
fn into_par_vec() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    test_string_owned(vec.into_par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    test_numbers_owned(vec.into_par(), INPUT_LEN);
}

#[test]
fn into_par_slice() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    let slice = vec.as_slice();
    test_string_ref(slice.into_par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    let slice = vec.as_slice();
    test_numbers_ref(slice.into_par(), INPUT_LEN);
}

#[test]
fn into_par_iter() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    let iter = vec
        .iter()
        .map(|x| (x.len(), x))
        .filter(|x| x.0 < 100)
        .take(INPUT_LEN)
        .map(|x| x.1)
        .into_con_iter();
    test_string_ref(iter.into_par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    let iter = vec
        .iter()
        .map(|x| (2 * x, x))
        .filter(|x| x.0 < 3 * INPUT_LEN)
        .take(INPUT_LEN)
        .map(|x| x.1)
        .into_con_iter();
    test_numbers_ref(iter.into_par(), INPUT_LEN);

    let iter = (0..(2 * INPUT_LEN))
        .filter(|x| x < &INPUT_LEN)
        .map(|i| i.to_string())
        .take(INPUT_LEN)
        .into_con_iter();
    test_string_owned(iter.into_par(), INPUT_LEN);

    let iter = (0..(2 * INPUT_LEN))
        .filter(|x| x < &INPUT_LEN)
        .take(INPUT_LEN)
        .into_con_iter();
    test_numbers_owned(iter.into_par(), INPUT_LEN);
}

#[test]
fn into_par_iter_owned() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    let iter = vec
        .into_iter()
        .map(|x| (x.len(), x))
        .filter(|x| x.0 < 100)
        .take(INPUT_LEN)
        .map(|x| x.1)
        .into_con_iter();
    test_string_owned(iter.into_par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    let iter = vec
        .into_iter()
        .map(|x| (2 * x, x))
        .filter(|x| x.0 < 3 * INPUT_LEN)
        .take(INPUT_LEN)
        .map(|x| x.1)
        .into_con_iter();
    test_numbers_owned(iter.into_par(), INPUT_LEN);
}

#[test]
fn into_par_iter_range() {
    let range = 0usize..INPUT_LEN;
    test_numbers_owned(range.into_par(), INPUT_LEN);
}
