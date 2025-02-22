use crate::into::core::*;
use orx_parallel::*;

const INPUT_LEN: usize = 1024;

#[test]
fn into_par_iter_range() {
    let range = 0usize..INPUT_LEN;
    test_numbers_owned(range.par(), INPUT_LEN);
}

#[test]
fn into_par_iter() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    let iter = vec
        .iter()
        .map(|x| (x.len(), x))
        .filter(|x| x.0 < 100)
        .take(INPUT_LEN)
        .map(|x| x.1);
    test_string_ref(iter.par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    let iter = vec
        .iter()
        .map(|x| (2 * x, x))
        .filter(|x| x.0 < 3 * INPUT_LEN)
        .take(INPUT_LEN)
        .map(|x| x.1);
    test_numbers_ref(iter.par(), INPUT_LEN);

    let iter = (0..(2 * INPUT_LEN))
        .filter(|x| x < &INPUT_LEN)
        .map(|i| i.to_string())
        .take(INPUT_LEN);
    test_string_owned(iter.par(), INPUT_LEN);

    let iter = (0..(2 * INPUT_LEN))
        .filter(|x| x < &INPUT_LEN)
        .take(INPUT_LEN);
    test_numbers_owned(iter.par(), INPUT_LEN);
}

#[test]
fn into_par_iter_owned() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    let iter = vec
        .into_iter()
        .map(|x| (x.len(), x))
        .filter(|x| x.0 < 100)
        .take(INPUT_LEN)
        .map(|x| x.1);
    test_string_owned(iter.par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    let iter = vec
        .into_iter()
        .map(|x| (2 * x, x))
        .filter(|x| x.0 < 3 * INPUT_LEN)
        .take(INPUT_LEN)
        .map(|x| x.1);
    test_numbers_owned(iter.par(), INPUT_LEN);
}
