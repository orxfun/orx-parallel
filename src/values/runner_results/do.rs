pub enum ThreadDo<E> {
    Done,
    StoppedByWhileCondition { idx: usize },
    StoppedByError { idx: usize, error: E },
}
