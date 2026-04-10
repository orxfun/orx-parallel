use crate::collectables::algorithms::merge_ordered_vectors::slice_iter_ptr_dst::SliceIterPtrDst;

use super::slice_src::SliceSrc;

/// Determines the streak search.
///
/// Assume at an intermediate step of the algorithm, current value of left slice is
/// less than current value of right slice.
///
/// In this case, we will copy the current element of the left slice to the current
/// position of the target slice.
///
/// However, if next `n` elements of the left slice are all smaller than the current
/// value of the right slice, we can copy all `n` elements at once. The subslice of
/// this `n` elements is called the streak.
#[derive(Clone, Copy, Debug)]
pub enum StreakSearch {
    /// We don't search for a streak; we copy elements one by one.
    None,
    /// We search the streak by a linear search.
    Linear,
    /// We search the streak by a binary search since both source slices are sorted.
    Binary,
}

/// Parameters of the sequential algorithm for merging two sorted slices into one sorted slice.
#[derive(Clone, Copy, Debug)]
pub struct ParamsSeqMergeSortedSlices {
    /// Streak search method.
    pub streak_search: StreakSearch,
    /// When true, the algorithm always puts the larger slice to the left;
    /// otherwise to the right.
    pub put_large_to_left: bool,
}

fn seq_merge_streak_none<'a, T: 'a, D>(
    mut left: SliceSrc<'a, T>,
    mut right: SliceSrc<'a, T>,
    target: D,
    put_large_to_left: bool,
) where
    D: SliceIterPtrDst<'a, T>,
{
    // let is_large_on_left = left.len() >= right.len();
    // if is_large_on_left != put_large_to_left {
    //     (left, right) = (right, left);
    // }
}
