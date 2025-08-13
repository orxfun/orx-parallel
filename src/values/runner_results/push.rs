pub enum ValuesPush<E> {
    Done,
    StoppedByWhileCondition { idx: usize },
    StoppedByError { idx: usize, error: E },
}
