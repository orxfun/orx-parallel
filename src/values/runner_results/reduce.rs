use crate::values::{Values, runner_results::Fallibility};

pub enum Reduce<V: Values> {
    Done {
        acc: V::Item,
    },
    StoppedByWhileCondition,
    StoppedByError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}
