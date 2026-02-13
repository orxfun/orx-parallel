pub enum StreakSearch {
    None,
    Linear,
    Binary,
}

pub enum SplitPivotSearch {
    Linear,
    Binary,
}

pub struct ParamsParMergeSortedSlices {
    pub num_threads: usize,
    pub seq_threshold: usize,
    pub pivot_search: SplitPivotSearch,
    pub put_large_to_left: bool,
    pub seq_params: ParamsSeqMergeSortedSlices,
}

pub struct ParamsSeqMergeSortedSlices {
    pub streak_search: StreakSearch,
    pub put_large_to_left: bool,
}
