use crate::experiment::data_structures::slice::Slice;
use alloc::vec;

#[test]
fn slice_overlap() {
    let a = vec![1, 2, 3, 4, 5, 6];
    let b = vec![7, 8];

    let assert_no_overlap = |x: &[i32], y: &[i32]| {
        assert!(Slice::from(x).is_non_overlapping(&Slice::from(y)));
        assert!(Slice::from(y).is_non_overlapping(&Slice::from(x)));
    };

    let assert_overlap = |x: &[i32], y: &[i32]| {
        assert!(!Slice::from(x).is_non_overlapping(&Slice::from(y)));
        assert!(!Slice::from(y).is_non_overlapping(&Slice::from(x)));
    };

    assert_no_overlap(&a[..], &b[..]);
    assert_no_overlap(&a[0..2], &a[4..]);
    assert_no_overlap(&a[1..3], &a[3..6]);
    assert_no_overlap(&a[0..0], &a[..]);

    assert_overlap(&a[..], &a[..]);
    assert_overlap(&a[0..2], &a[1..3]);
    assert_overlap(&a[0..2], &a[1..2]);
}
