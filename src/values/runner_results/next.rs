use crate::values::{Values, runner_results::Fallibility};

pub enum Next<V: Values> {
    Done {
        value: Option<V::Item>,
    },
    StoppedByWhileCondition,
    StoppedByError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}

pub enum NextWithIdx<V: Values> {
    Found {
        value: V::Item,
        idx: usize,
    },
    NotFound,
    StoppedByWhileCondition {
        idx: usize,
    },
    StoppedByError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}
