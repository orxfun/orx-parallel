use super::sequential;
use crate::algorithms::data_structures::{Slice, SliceMut};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum StreakSearch {
    None,
    Linear,
    Binary,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum PivotSearch {
    Linear,
    Binary,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct MergeSortedSlicesParams {
    pub num_threads: usize,
    pub streak_search: StreakSearch,
    pub sequential_merge_threshold: usize,
    pub pivot_search: PivotSearch,
    pub put_large_to_left: bool,
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
