use super::sequential;
use crate::algorithms::data_structures::{Slice, SliceMut};

pub fn merge_sorted_slices<T, F>(
    is_leq: F,
    left: &[T],
    right: &[T],
    target: &mut [T],
    num_threads: usize,
) where
    F: Fn(&T, &T) -> bool,
{
    assert_eq!(left.len() + right.len(), target.len());

    let left = Slice::from(left);
    let right = Slice::from(right);
    let mut target = SliceMut::from(target);

    match num_threads {
        1 => sequential::merge_sorted_slices(is_leq, &left, &right, &mut target),
        _ => todo!(),
    }
}
