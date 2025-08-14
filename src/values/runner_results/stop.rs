pub enum Stop<E> {
    DueToWhile,
    DueToError { error: E },
}

pub enum StopWithIdx<E> {
    DueToWhile { idx: usize },
    DueToError { idx: usize, error: E },
}
