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
