use crate::generic_values::runner_results::Fallibility;

pub enum SequentialPush<F: Fallibility> {
    Done,
    StoppedByWhileCondition,
    StoppedByError { error: F::Error },
}
