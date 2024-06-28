use crate::into::core::*;
use orx_parallel::*;

const INPUT_LEN: usize = 1024;

#[test]
fn as_par_vec() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    test_string_ref(vec.par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    test_numbers_ref(vec.par(), INPUT_LEN);
}

#[test]
fn as_par_slice() {
    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i.to_string()).collect();
    let slice = vec.as_slice();
    test_string_ref(slice.par(), INPUT_LEN);

    let vec: Vec<_> = (0..INPUT_LEN).map(|i| i).collect();
    let slice = vec.as_slice();
    test_numbers_ref(slice.par(), INPUT_LEN);
}

#[test]
fn as_par_array() {
    let mut array = [0usize; INPUT_LEN];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    test_numbers_ref(array.par(), INPUT_LEN);
}
