pub trait ParallelTaskWithIdx {
    type Item;

    fn f1(&self, idx: usize, value: Self::Item);

    fn fc(&self, begin_idx: usize, values: impl ExactSizeIterator<Item = Self::Item>);
}
