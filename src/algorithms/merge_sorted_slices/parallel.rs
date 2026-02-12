use crate::algorithms::data_structures::Slice;
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

struct TaskData<'a, T> {
    left: Slice<'a, T>,
    right: Slice<'a, T>,
    target: Slice<'a, T>,
}

enum Task {
    CopyOneSide,
    SequentialMerge,
    Split,
}
