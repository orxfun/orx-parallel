pub trait Use: Sync {
    type Item;

    fn create(&self, thread_idx: usize) -> Self::Item;
}
