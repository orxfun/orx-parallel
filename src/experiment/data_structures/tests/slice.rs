use crate::experiment::data_structures::slice::{Slice, SliceSafe};
use alloc::vec;
use alloc::vec::Vec;
use std::string::ToString;
use test_case::test_matrix;

#[test]
fn slice_overlap() {
    let a = vec![1, 2, 3, 4, 5, 6];
    let b = vec![7, 8];

    let assert_no_overlap = |x: &[i32], y: &[i32]| {
        let [x, y] = [x, y].map(Slice::from);
        let [x, y] = [&x, &y].map(SliceSafe::from);
        assert!(x.is_non_overlapping(&y));
        assert!(y.is_non_overlapping(&x));
    };

    let assert_overlap = |x: &[i32], y: &[i32]| {
        let [x, y] = [x, y].map(Slice::from);
        let [x, y] = [&x, &y].map(SliceSafe::from);
        assert!(!x.is_non_overlapping(&y));
        assert!(!y.is_non_overlapping(&x));
    };

    assert_no_overlap(&a[..], &b[..]);
    assert_no_overlap(&a[0..2], &a[4..]);
    assert_no_overlap(&a[1..3], &a[3..6]);
    assert_no_overlap(&a[0..0], &a[..]);

    assert_overlap(&a[..], &a[..]);
    assert_overlap(&a[0..2], &a[1..3]);
    assert_overlap(&a[0..2], &a[1..2]);
}

#[test_matrix([0, 1, 2, 3, 6])]
fn slice_split_unchecked(len: usize) {
    let vec: Vec<_> = (1..(1 + len)).map(|x| x.to_string()).into_iter().collect();

    for i in 0..=vec.len() {
        let s = Slice::from(vec.as_slice());

        // SAFETY: (i) i <= s.len()
        let [l, r] = unsafe { s.split_at_unchecked(i) };

        // SAFETY: all initialized & no mutation
        let left = unsafe { l.as_slice() }.to_vec();
        let right = unsafe { r.as_slice() }.to_vec();
        assert_eq!(&left, &vec[0..i]);
        assert_eq!(&right, &vec[i..]);
    }
}
