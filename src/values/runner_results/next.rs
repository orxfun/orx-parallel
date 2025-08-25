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
        idx: usize,
        value: V::Item,
    },
    NotFound,
    StoppedByWhileCondition {
        idx: usize,
    },
    StoppedByError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}

pub enum NextSuccess<T> {
    Found { idx: usize, value: T },
    StoppedByWhileCondition { idx: usize },
}

impl<T> NextSuccess<T> {
    pub fn reduce(results: Vec<Self>) -> Option<(usize, T)> {
        let mut result = None;
        let mut idx_bound = usize::MAX;
        for x in results {
            match x {
                NextSuccess::Found { idx, value } if idx < idx_bound => {
                    idx_bound = idx;
                    result = Some((idx, value));
                }
                NextSuccess::StoppedByWhileCondition { idx } if idx < idx_bound => {
                    idx_bound = idx;
                }
                _ => {}
            }
        }

        result.and_then(|(idx, value)| match idx <= idx_bound {
            true => Some((idx, value)),
            false => None, // found value was found after stopped
        })
    }
}
