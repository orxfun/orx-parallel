pub trait Use: Sync {
    type Item;

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item;

    fn get(&mut self, thread_idx: usize) -> &mut Self::Item;

    fn max_threads(&self) -> Option<usize>;
}
