use crate::generic_values::runner_results::{
    Fallibility, Fallible, Infallible, Stop, fallibility::Never,
};

pub enum SequentialPush<F: Fallibility> {
    Done,
    StoppedByWhileCondition,
    StoppedByError { error: F::Error },
}

impl SequentialPush<Infallible> {
    pub fn sequential_push_to_stop(self) -> Option<Stop<Never>> {
        match self {
            SequentialPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            _ => None,
        }
    }
}

impl<E: Send> SequentialPush<Fallible<E>> {
    pub fn sequential_push_to_stop(self) -> Option<Stop<E>> {
        match self {
            SequentialPush::Done => None,
            SequentialPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            SequentialPush::StoppedByError { error } => Some(Stop::DueToError { error }),
        }
    }
}
