pub struct State {
    pub max_num_threads: usize,
    pub size_hint: (usize, Option<usize>),
    pub chunk_size: usize,
}
