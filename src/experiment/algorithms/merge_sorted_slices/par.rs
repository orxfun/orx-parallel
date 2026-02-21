use crate::experiment::algorithms::merge_sorted_slices::seq::{
    ParamsSeqMergeSortedSlices, seq_merge_unchecked,
};
use crate::experiment::data_structures::{slice_dst::SliceDst, slice_src::SliceSrc};
use crate::{IntoParIterRec, ParIter};
use orx_concurrent_recursive_iter::Queue;

/// Parameters of the sequential algorithm for merging two sorted slices into one sorted slice.
#[derive(Clone, Copy, Debug)]
pub struct ParamsParMergeSortedSlices {
    /// Parameters of sequential merging.
    pub seq_params: ParamsSeqMergeSortedSlices,
    /// When true, the algorithm always puts the larger slice to the left;
    /// otherwise to the right.
    pub put_large_to_left: bool,
    /// Number of threads.
    pub num_threads: usize,
    /// Chunk size to be used by parallelization.
    pub chunk_size: usize,
}

struct Task<'a, T> {
    left: SliceSrc<'a, T>,
    right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
}

unsafe impl<'a, T> Send for Task<'a, T> {}

impl<'a, T> Task<'a, T> {
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
pub fn par_merge<'a, T: 'a, F>(
    is_leq: F,
    left: SliceSrc<'a, T>,
    right: SliceSrc<'a, T>,
    target: SliceDst<'a, T>,
    params: ParamsParMergeSortedSlices,
) where
    T: Send + Sync,
    F: Fn(&T, &T) -> bool + Send + Sync,
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
        unsafe { handle_extend(&is_leq, &params, task, queue) }
    };

    initial_task
        .into_par_rec(handle_extend)
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
    t: &Task<'a, T>,
    queue: &Queue<'_, Task<'a, T>>,
) where
    T: Send + Sync,
    F: Fn(&T, &T) -> bool + Send + Sync,
{
    let t = unsafe { t.clone() };
    let (mut left, mut right, target) = (t.left, t.right, t.target);
    match (left.len(), right.len()) {
        (x, _) if x < 3 => {
            // SAFETY: req't (i) & (ii) are satisfied by conditions (i) & (ii)
            unsafe { seq_merge_unchecked(is_leq, left, right, target, &params.seq_params) };
        }
        (_, x) if x < 3 => {
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

            let pos_right = unsafe { right.as_slice() }
                .iter()
                .position(|r| is_leq(pivot, r))
                .unwrap_or(right.len());
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
