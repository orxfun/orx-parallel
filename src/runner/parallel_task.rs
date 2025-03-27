pub trait ParallelTask {
    type Item;

    fn f1(&self, value: Self::Item);

    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>);
}

pub trait ParallelTaskWithIdx {
    type Item;

    fn f1(&self, idx: usize, value: Self::Item);

    fn fc(&self, begin_idx: usize, values: impl ExactSizeIterator<Item = Self::Item>);
}
