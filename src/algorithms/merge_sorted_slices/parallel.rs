use crate::algorithms::data_structures::Slice;
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

struct Task<'a, T> {
    left: Slice<'a, T>,
    right: Slice<'a, T>,
    target: Slice<'a, T>,
}

impl<'a, T> Task<'a, T> {
    fn do_merge_sequentially(&self, sequential_merge_threshold: usize) -> bool {
        self.left.len() < 2
            || self.right.len() < 2
            || self.left.len() + self.right.len() <= sequential_merge_threshold
    }

    // sequential runs

    fn copy_left_to_target(&self) {
        debug_assert_eq!(self.right.len(), 0);
        self.target.copy_from_nonoverlapping(&self.left);
    }

    fn copy_right_to_target(&self) {
        debug_assert_eq!(self.left.len(), 0);
        self.target.copy_from_nonoverlapping(&self.right);
    }
}

impl<'a, T> Clone for Task<'a, T> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            target: self.target.clone(),
        }
    }
}

fn handle<'a, 'b, T>(t: &'a Task<'b, T>, queue: &Queue<'b, Task<'b, T>>)
where
    T: Send,
{
    match (t.left.len(), t.right.len()) {
        (0, _) => t.copy_right_to_target(),
        (_, 0) => t.copy_left_to_target(),
        _ => {
            //
            todo!()
        }
    }
}
