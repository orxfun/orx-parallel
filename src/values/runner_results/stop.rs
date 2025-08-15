use crate::values::{Values, runner_results::Fallibility};

pub enum Stop<E> {
    DueToWhile,
    DueToError { error: E },
}

pub enum StopWithIdx<E> {
    DueToWhile { idx: usize },
    DueToError { idx: usize, error: E },
}

pub enum StopReduce<V: Values> {
    DueToWhile {
        acc: Option<V::Item>,
    },
    DueToError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}
