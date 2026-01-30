use crate::algorithms::data_structures::{slice::Slice, slice_mut::SliceMut};
use alloc::vec;

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

#[test]
fn slice_iter_over_ref() {
    let vec = vec![0, 1, 2, 3, 4, 5, 6];

    let slice = Slice::from(vec.as_slice());

    let mut iter = slice.iter_over_ref();
    for i in 0..vec.len() {
        assert_eq!(iter.peek(), Some(&i));
        assert_eq!(iter.next(), Some(&i));
    }

    assert_eq!(iter.peek(), None);
    assert_eq!(iter.next(), None);

    assert_eq!(iter.peek(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn slice_iter_as_dst_write_one_unchecked() {
    let src_vec = vec![0, 1, 2, 3, 4, 5];
    let src_slice = Slice::from(src_vec.as_slice());

    let mut dst_vec = vec![42; src_vec.len()];
    let mut slice = SliceMut::from(dst_vec.as_mut_slice());
    let mut dst = slice.iter_as_dst();
    for src in src_slice.iter_over_ptr() {
        unsafe { dst.write_one_unchecked(src) };
    }

    assert_eq!(&src_vec, &dst_vec);
}

#[test]
fn slice_iter_as_dst_write_remaining() {
    let src_vec = vec![0, 1, 2, 3, 4, 5];
    let src_slice = Slice::from(src_vec.as_slice());

    let mut dst_vec = vec![42; src_vec.len()];
    let mut slice = SliceMut::from(dst_vec.as_mut_slice());
    let mut dst = slice.iter_as_dst();
    unsafe { dst.write_remaining_from(&src_slice) };

    assert_eq!(&src_vec, &dst_vec);
}

#[test]
fn slice_iter_as_dst_write_remaining_halfway() {
    let src_vec1 = vec![0, 1];
    let src_slice1 = Slice::from(src_vec1.as_slice());
    let src_vec2 = vec![2, 3, 4, 5];
    let src_slice2 = Slice::from(src_vec2.as_slice());

    let mut dst_vec = vec![42; src_vec1.len() + src_vec2.len()];
    let mut slice = SliceMut::from(dst_vec.as_mut_slice());
    let mut dst = slice.iter_as_dst();
    for src in src_slice1.iter_over_ptr() {
        unsafe { dst.write_one_unchecked(src) };
    }
    unsafe { dst.write_remaining_from(&src_slice2) };

    assert_eq!(&vec![0, 1, 2, 3, 4, 5], &dst_vec);
}
