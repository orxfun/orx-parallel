pub struct State {
    pub max_num_threads: usize,
    pub initial_len: Option<usize>,
    pub chunk_size: usize,
    /// Minimum number of queue items that must be visible before spawning the next thread.
    /// Computed as `chunk_size * min_items_per_thread_factor`.
    pub min_items_per_thread: usize,
}
