pub trait Use: Sync {
    type Item;

    fn create(&self, thread_idx: usize) -> Self::Item;

    fn into_inner(self) -> Self::Item;
}
