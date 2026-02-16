use crate::experiment::{
    algorithms::merge_sorted_slices::seq::{ParamsSeqMergeSortedSlices, seq_merge_unchecked},
    data_structures::{slice_dst::SliceDst, slice_src::SliceSrc},
};
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

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

fn handle<'a, 'b, T: 'a, F>(
    is_leq: F,
    t: &'a Task<'b, T>,
    queue: &Queue<'b, Task<'b, T>>,
    params: &ParamsSeqMergeSortedSlices,
) where
    T: Send,
    F: Fn(&T, &T) -> bool,
{
    let t = unsafe { t.clone() };
    match (t.left.len(), t.right.len()) {
        (0, _) | (_, 0) => {
            unsafe { seq_merge_unchecked(is_leq, t.left, t.right, t.target, params) };
        }
        _ => {}
    }
}
