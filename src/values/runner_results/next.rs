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

impl<V: Values> NextWithIdx<V> {
    fn found_or_stopped_idx(&self) -> Option<usize> {
        match self {
            Self::Found { idx, value: _ } => Some(*idx),
            Self::StoppedByWhileCondition { idx } => Some(*idx),
            _ => None,
        }
    }

    pub fn into_found_value(self) -> Option<V::Item> {
        match self {
            Self::Found { idx: _, value } => Some(value),
            _ => None,
        }
    }

    /// Returns the value with the smallest found idx whose idx is less than
    /// the smallest of the stopped indices, if any.
    ///
    /// Returns None if there is no found items before the process stopped.
    pub fn reduce(results: Vec<Self>) -> Self {
        let mut result = Self::NotFound;
        let mut idx_bound = usize::MAX;
        for x in results {
            if let Some(idx) = x.found_or_stopped_idx()
                && idx < idx_bound
            {
                idx_bound = idx;
                result = x;
            }
        }

        result
    }
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
