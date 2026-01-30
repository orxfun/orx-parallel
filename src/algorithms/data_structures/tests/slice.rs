use crate::algorithms::data_structures::slice::Slice;
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn slice_get() {
    let vec = vec![0, 1, 2, 3, 4, 5, 6];
    let slice = Slice::from(vec.as_slice());

    assert_eq!(vec.len(), slice.len());

    for i in 0..slice.len() {
        assert_eq!(slice.get(i), Some(&i));
    }
    for i in slice.len()..2 * slice.len() {
        assert_eq!(slice.get(i), None);
    }
}
