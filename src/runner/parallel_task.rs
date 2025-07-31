/// A parallel task shared with threads.
pub trait ParallelTask {
    /// Item.
    type Item;

    /// Task to perform on a single `value`.
    fn f1(&self, value: Self::Item);

    /// Task to perform on a chunk of `values` with known length.
    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>);
}

/// A parallel task shared with threads, where indices are also used.
pub trait ParallelTaskWithIdx: Clone {
    /// Item.
    type Item;

    /// Task to perform on a single `value` with the given input `idx`.
    fn f1(&self, idx: usize, value: Self::Item);

    /// Task to perform on a chunk of `values` with known length starting at the given `begin_idx`.
    fn fc(&self, begin_idx: usize, values: impl ExactSizeIterator<Item = Self::Item>);
}
