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

impl<V: Values> core::fmt::Debug for Reduce<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Done { acc: _ } => f.debug_struct("Done").finish(),
            Self::StoppedByWhileCondition { acc: _ } => {
                f.debug_struct("StoppedByWhileCondition").finish()
            }
            Self::StoppedByError { error: _ } => f.debug_struct("StoppedByError").finish(),
        }
    }
}
