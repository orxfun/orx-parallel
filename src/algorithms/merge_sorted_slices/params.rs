pub enum StreakSearch {
    None,
    Linear,
    Binary,
}

pub enum PivotSearch {
    Linear,
    Binary,
}

pub struct ExpParMergeSortedSlicesParams {
    pub num_threads: usize,
    pub sequential_merge_threshold: usize,
    pub pivot_search: PivotSearch,
    pub put_large_to_left: bool,
    pub seq_params: ExpSeqMergeSortedSlicesParams,
}

pub struct ExpSeqMergeSortedSlicesParams {
    pub streak_search: StreakSearch,
    pub put_large_to_left: bool,
}
