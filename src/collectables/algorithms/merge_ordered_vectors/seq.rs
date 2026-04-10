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
    mut dst: D,
    put_large_to_left: bool,
) where
    D: SliceIterPtrDst<'a, T>,
{
    let is_large_on_left = left.len() >= right.len();
    if is_large_on_left != put_large_to_left {
        (left, right) = (right, left);
    }

    let mut left = left.into_ptr_iter();
    let mut right = right.into_ptr_iter();

    match (left.current_idx(), right.current_idx()) {
        // (Some(mut l), Some(mut r)) => {
        //     loop {
        //         match is_leq(l, r) {
        //             true => {
        //                 // SAFETY: left still has at least one elem `l`, so must `dst`
        //                 unsafe { dst.write_one_from(&mut left) };
        //                 match left.current() {
        //                     Some(x) => l = x,
        //                     None => {
        //                         // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
        //                         unsafe { dst.write_rest_from(&mut right) };
        //                         break;
        //                     }
        //                 }
        //             }
        //             false => {
        //                 // SAFETY: right still has at least one elem `r`, so must `dst`
        //                 unsafe { dst.write_one_from(&mut right) };

        //                 match right.current() {
        //                     Some(x) => r = x,
        //                     None => {
        //                         // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
        //                         unsafe { dst.write_rest_from(&mut left) };
        //                         break;
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        (None, None) => {}
        (None, _) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(right) };
        }
        // (_, None) => {
        //     // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
        //     unsafe { dst.write_rest_from(&mut left) };
        // }
        _ => todo!(),
    }
}
