/// A parallel task shared with threads.
pub trait ParallelTask {
    /// Item.
    type Item;

    /// Task to perform on a single `value`.
    fn f1(&self, value: Self::Item);

    /// Task to perform on a chunk of `values` with known length.
    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>);
}
