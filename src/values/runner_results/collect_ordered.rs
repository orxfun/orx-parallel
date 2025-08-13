pub enum ValuesPush<E> {
    Done,
    StoppedByWhileCondition { idx: usize },
    StoppedByError { idx: usize, error: E },
}

pub enum ThreadCollect<T, E> {
    AllCollected {
        vec: Vec<(usize, T)>,
    },
    StoppedByWhileCondition {
        vec: Vec<(usize, T)>,
        stopped_idx: usize,
    },
    StoppedByError {
        error: E,
    },
}
