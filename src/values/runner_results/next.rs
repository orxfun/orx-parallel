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

pub struct NextWithIdx<V: Values> {
    next: Next<V>,
    idx: usize,
}

impl<V: Values> Next<V> {
    pub fn with_idx(self, idx: usize) -> NextWithIdx<V> {
        NextWithIdx { next: self, idx }
    }
}
