use super::sequential;
use crate::algorithms::data_structures::{Slice, SliceMut};

pub struct MergeSortedSlicesParams {
    pub with_streaks: bool,
    pub num_threads: usize,
}

pub fn merge_sorted_slices<T, F>(
    is_leq: F,
    left: &[T],
    right: &[T],
    target: &mut [T],
    params: MergeSortedSlicesParams,
) where
    F: Fn(&T, &T) -> bool,
{
    assert_eq!(left.len() + right.len(), target.len());

    let left = Slice::from(left);
    let right = Slice::from(right);
    let mut target = SliceMut::from(target);

    match params.num_threads {
        1 => sequential::merge_sorted_slices(is_leq, &left, &right, &mut target, params),
        _ => todo!(),
    }
}
