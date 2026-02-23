use crate::experiment::algorithms::merge_sorted_slices::seq::{
    ParamsSeqMergeSortedSlices, bin_search_idx, seq_merge_unchecked,
};
use crate::experiment::data_structures::{slice_dst::SliceDst, slice_src::SliceSrc};
use crate::{IntoParIterRec, ParIter, ParallelRunner};
use core::cmp::Ordering;
use orx_concurrent_recursive_iter::Queue;

/// Determines how to search the pivot for splitting the slices.
#[derive(Clone, Copy, Debug)]
pub enum PivotSearch {
    /// We search the pivot position by a linear search.
    Linear,
    /// We search the pivot position by a binary search since both source slices are sorted.
    Binary,
}

/// Parameters of the sequential algorithm for merging two sorted slices into one sorted slice.
#[derive(Clone, Copy, Debug)]
pub struct ParamsParMergeSortedSlices {
    /// Parameters of sequential merging.
    pub seq_params: ParamsSeqMergeSortedSlices,
    /// When true, the algorithm always puts the larger slice to the left;
    /// otherwise to the right.
    pub put_large_to_left: bool,
    /// Determines how to search the pivot for splitting the slices.
    pub pivot_search: PivotSearch,
    /// Number of threads.
    pub num_threads: usize,
    /// Chunk size to be used by parallelization.
    pub chunk_size: usize,
    /// Minimum length of a slice to be split into two tasks.
    pub min_split_len: usize,
}

struct Task<'a, T> {
    left: SliceSrc<'a, T>,
    right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
}

unsafe impl<'a, T> Send for Task<'a, T> {}

impl<'a, T> Task<'a, T> {
    /// Clones the task.
    ///
    /// # SAFETY
    ///
    /// The purpose of destination slice `target` is to mutate the underlying memory.
    /// Therefore, cloning task is marked as unsafe.
    ///
    /// - (i) assuming the clone will be used to mutate the memory, caller must
    ///   ensure that `&self.target` will not be used.
    unsafe fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            target: unsafe { self.target.clone() },
        }
    }
}

/// # Panics
///
/// - (i) if `target.len()` is not equal to `left.len() + right.len()`
/// - (ii) if any pair of of `left`, `right` or `target` are overlapping.
pub fn par_merge<'a, T, F, R: ParallelRunner>(
    is_leq: F,
    left: SliceSrc<'a, T>,
    right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    params: &ParamsParMergeSortedSlices,
    runner: R,
) where
    T: Send + Sync,
    F: Fn(&T, &T) -> bool + Sync,
{
    assert_eq!(target.len(), left.len() + right.len());
    assert!(target.core().is_non_overlapping(&left.core()));
    assert!(target.core().is_non_overlapping(&right.core()));
    assert!(left.core().is_non_overlapping(&right.core()));

    let initial_task = [Task {
        left,
        right,
        target,
    }];

    let handle_extend = |task: &Task<'a, T>, queue: &Queue<'_, Task<'a, T>>| {
        // SAFETY: req't (i) and (ii) are satisfied by panic conditions
        unsafe { handle_extend(&is_leq, params, task, queue) }
    };

    initial_task
        .into_par_rec(handle_extend)
        .with_runner(runner)
        .num_threads(params.num_threads)
        .chunk_size(params.chunk_size)
        .for_each(|_| {});
}

/// # SAFETY
///
/// - (i) `target.len()` must equal `left.len() + right.len()`
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
unsafe fn handle_extend<'a, T, F>(
    is_leq: F,
    params: &ParamsParMergeSortedSlices,
    task: &Task<'a, T>,
    queue: &Queue<'_, Task<'a, T>>,
) where
    T: Send + Sync,
    F: Fn(&T, &T) -> bool,
{
    let min_split_len = core::cmp::min(params.min_split_len, 3);

    // SAFETY: this method both handles and extends the queue; which will be
    // visited only once; hence, the reference `task` will not be used to
    // mutate the underlying memory, satisfying condition (i).
    let task = unsafe { task.clone() };

    let (mut left, mut right, target) = (task.left, task.right, task.target);
    match (left.len(), right.len()) {
        (x, y) if x < min_split_len || y < min_split_len => {
            // SAFETY: req't (i) & (ii) are satisfied by conditions (i) & (ii)
            unsafe { seq_merge_unchecked(is_leq, left, right, target, &params.seq_params) };
        }
        _ => {
            let is_large_on_left = left.len() >= right.len();
            if is_large_on_left != params.put_large_to_left {
                (left, right) = (right, left);
            }

            let position = left.len() / 2;
            // SAFETY: position <= self.len()
            let [left_left, left_right] = unsafe { left.split_at_unchecked(position) };

            // SAFETY: since left.len() >= 2, then left_right.len() > 0
            let pivot = unsafe { left_right.first_unchecked() };

            let pos_right =
                match params.pivot_search {
                    PivotSearch::Linear => right
                        .values()
                        .position(|r| is_leq(pivot, r))
                        .unwrap_or(right.len()),
                    PivotSearch::Binary => bin_search_idx(right.as_slice().binary_search_by(|r| {
                        match is_leq(r, pivot) {
                            true => Ordering::Less,
                            false => Ordering::Greater,
                        }
                    })),
                };

            // SAFETY: (i) pos_right <= right.len() is satisfied by the expression declaring pos_right
            let [right_left, right_right] = unsafe { right.split_at_unchecked(pos_right) };

            let target_left_len = left_left.len() + right_left.len();
            // SAFETY: (i) target_left_len <= target.len() is satisfied by cond'n (i) target.len() == left.len() + right.len()
            let [target_left, target_right] = unsafe { target.split_at_unchecked(target_left_len) };

            let task_left = Task {
                left: left_left,
                right: right_left,
                target: target_left,
            };
            let task_right = Task {
                left: left_right,
                right: right_right,
                target: target_right,
            };
            queue.extend([task_left, task_right]);
        }
    }
}
