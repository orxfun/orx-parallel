use crate::algorithms::data_structures::Slice;
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

enum Task<'a, T> {
    CopyOneSide {
        source: Slice<'a, T>,
        target: Slice<'a, T>,
    },

    SequentialMerge {
        left: Slice<'a, T>,
        right: Slice<'a, T>,
        target: Slice<'a, T>,
    },

    Split {
        left: Slice<'a, T>,
        right: Slice<'a, T>,
        target: Slice<'a, T>,
    },
}
