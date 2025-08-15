use crate::values::{Values, runner_results::Fallibility};

pub enum Reduce<V: Values> {
    Done {
        acc: Option<V::Item>,
    },
    StoppedByWhileCondition {
        acc: Option<V::Item>,
    },
    StoppedByError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}
