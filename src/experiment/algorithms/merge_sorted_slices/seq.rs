use crate::experiment::data_structures::{slice_dst::SliceDst, slice_src::SliceSrc};
use core::cmp::Ordering;

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
#[derive(Clone, Copy)]
pub enum StreakSearch {
    /// We don't search for a streak; we copy elements one by one.
    None,
    /// We search the streak by a linear search.
    Linear,
    /// We search the streak by a binary search since both source slices are sorted.
    Binary,
}

/// Parameters of the sequential algorithm for merging two sorted slices into one sorted slice.
#[derive(Clone, Copy)]
pub struct ParamsSeqMergeSortedSlices {
    /// Streak search method.
    pub streak_search: StreakSearch,
    /// When true, the algorithm always puts the larger slice to the left;
    /// otherwise to the right.
    pub put_large_to_left: bool,
}

/// # Panics
///
/// - (i) if `target.len()` is not equal to `left.len() + right.len()`
/// - (ii) if any pair of of `left`, `right` or `target` are overlapping.
pub fn seq_merge<'a, T: 'a, F>(
    is_leq: F,
    left: SliceSrc<'a, T>,
    right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    params: ParamsSeqMergeSortedSlices,
) where
    F: Fn(&T, &T) -> bool,
{
    assert_eq!(target.len(), left.len() + right.len());
    assert!(target.core().is_non_overlapping(&left.core()));
    assert!(target.core().is_non_overlapping(&right.core()));
    assert!(left.core().is_non_overlapping(&right.core()));

    // SAFETY: safety requirements are satisfied by panic conditions (i) and (ii)
    unsafe { seq_merge_unchecked(is_leq, left, right, target, params) }
}

/// # SAFETY
///
/// - (i) `target.len()` must equal `left.len() + right.len()`
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
pub unsafe fn seq_merge_unchecked<'a, T: 'a, F>(
    is_leq: F,
    left: SliceSrc<'a, T>,
    right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    params: ParamsSeqMergeSortedSlices,
) where
    F: Fn(&T, &T) -> bool,
{
    // SAFETY: safety requirements are satisfied by safety conditions (i) and (ii)
    unsafe {
        match params.streak_search {
            StreakSearch::None => {
                seq_merge_streak_none(is_leq, left, right, target, params.put_large_to_left)
            }
            StreakSearch::Linear => {
                seq_merge_streak_linear(is_leq, left, right, target, params.put_large_to_left)
            }
            StreakSearch::Binary => {
                seq_merge_streak_binary(is_leq, left, right, target, params.put_large_to_left)
            }
        }
    }
}

/// # SAFETY
///
/// - (i) `target.len()` must equal `left.len() + right.len()`
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
unsafe fn seq_merge_streak_none<'a, T: 'a, F>(
    is_leq: F,
    mut left: SliceSrc<'a, T>,
    mut right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    put_large_to_left: bool,
) where
    F: Fn(&T, &T) -> bool,
{
    let is_large_on_left = left.len() >= right.len();
    if is_large_on_left != put_large_to_left {
        (left, right) = (right, left);
    }

    let mut left = left.into_iter();
    let mut right = right.into_iter();
    let mut dst = target.into_iter();

    match (left.current(), right.current()) {
        (Some(mut l), Some(mut r)) => {
            loop {
                match is_leq(l, r) {
                    true => {
                        // SAFETY: left still has at least one elem `l`, so must `dst`
                        unsafe { dst.write_one_from(&mut left) };
                        match left.current() {
                            Some(x) => l = x,
                            None => {
                                // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
                                unsafe { dst.write_rest_from(&mut right) };
                                break;
                            }
                        }
                    }
                    false => {
                        // SAFETY: right still has at least one elem `r`, so must `dst`
                        unsafe { dst.write_one_from(&mut right) };

                        match right.current() {
                            Some(x) => r = x,
                            None => {
                                // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
                                unsafe { dst.write_rest_from(&mut left) };
                                break;
                            }
                        }
                    }
                }
            }
        }
        (None, None) => {}
        (None, _) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(&mut right) };
        }
        (_, None) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(&mut left) };
        }
    }
}

/// # SAFETY
///
/// - (i) `target.len()` must equal `left.len() + right.len()`
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
unsafe fn seq_merge_streak_linear<'a, T: 'a, F>(
    is_leq: F,
    mut left: SliceSrc<'a, T>,
    mut right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    put_large_to_left: bool,
) where
    F: Fn(&T, &T) -> bool,
{
    let is_large_on_left = left.len() >= right.len();
    if is_large_on_left != put_large_to_left {
        (left, right) = (right, left);
    }

    let mut left = left.into_iter();
    let mut right = right.into_iter();
    let mut dst = target.into_iter();

    match (left.current(), right.current()) {
        (Some(mut l), Some(mut r)) => {
            loop {
                match is_leq(l, r) {
                    true => {
                        let count = match left.values().position(|x| !is_leq(x, r)) {
                            Some(idx_bigger) => idx_bigger,
                            None => left.len(),
                        };
                        // SAFETY: left still has at least `count` elements, so must `dst`
                        unsafe { dst.write_many_from(&mut left, count) };

                        match left.current() {
                            Some(x) => l = x,
                            None => {
                                // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
                                unsafe { dst.write_rest_from(&mut right) };
                                break;
                            }
                        }
                    }
                    false => {
                        let count = match right.values().position(|x| !is_leq(x, l)) {
                            Some(idx_bigger) => idx_bigger,
                            None => right.len(),
                        };
                        // SAFETY: right still has at least `count` elements, so must `dst`
                        unsafe { dst.write_many_from(&mut right, count) };

                        match right.current() {
                            Some(x) => r = x,
                            None => {
                                // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
                                unsafe { dst.write_rest_from(&mut left) };
                                break;
                            }
                        }
                    }
                }
            }
        }
        (None, _) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(&mut right) };
        }
        (_, None) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(&mut left) };
        }
    }
}

/// # SAFETY
///
/// - (i) `target.len()` must equal `left.len() + right.len()`
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
unsafe fn seq_merge_streak_binary<'a, T: 'a, F>(
    is_leq: F,
    mut left: SliceSrc<'a, T>,
    mut right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    put_large_to_left: bool,
) where
    F: Fn(&T, &T) -> bool,
{
    let is_large_on_left = left.len() >= right.len();
    if is_large_on_left != put_large_to_left {
        (left, right) = (right, left);
    }

    let mut left = left.into_iter();
    let mut right = right.into_iter();
    let mut dst = target.into_iter();

    fn bin_search_idx(idx: Result<usize, usize>) -> usize {
        match idx {
            Ok(x) => x,
            Err(x) => x,
        }
    }

    match (left.current(), right.current()) {
        (Some(mut l), Some(mut r)) => {
            loop {
                match is_leq(l, r) {
                    true => {
                        let count =
                            bin_search_idx(left.as_slice().binary_search_by(|x| {
                                match is_leq(x, r) {
                                    true => Ordering::Less,
                                    false => Ordering::Greater,
                                }
                            }));
                        // SAFETY: left still has at least `count` elements, so must `dst`
                        unsafe { dst.write_many_from(&mut left, count) };

                        match left.current() {
                            Some(x) => l = x,
                            None => {
                                // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
                                unsafe { dst.write_rest_from(&mut right) };
                                break;
                            }
                        }
                    }
                    false => {
                        let count = bin_search_idx(right.as_slice().binary_search_by(|x| {
                            match is_leq(x, l) {
                                true => Ordering::Less,
                                false => Ordering::Greater,
                            }
                        }));
                        // SAFETY: right still has at least `count` elements, so must `dst`
                        unsafe { dst.write_many_from(&mut right, count) };

                        match right.current() {
                            Some(x) => r = x,
                            None => {
                                // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
                                unsafe { dst.write_rest_from(&mut left) };
                                break;
                            }
                        }
                    }
                }
            }
        }
        (None, _) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(&mut right) };
        }
        (_, None) => {
            // SAFETY: target (i) and (ii) are satisfied by conditions (i) and (ii)
            unsafe { dst.write_rest_from(&mut left) };
        }
    }
}
