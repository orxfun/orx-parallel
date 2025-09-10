use crate::ParallelRunner;

pub trait Orchestrator {
    type Runner: ParallelRunner;
}
