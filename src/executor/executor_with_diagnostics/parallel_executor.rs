use crate::ParallelExecutor;

pub struct ParallelExecutorWithDiagnostics<E>
where
    E: ParallelExecutor,
{
    executor: E,
}
