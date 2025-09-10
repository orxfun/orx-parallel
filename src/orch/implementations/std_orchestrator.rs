use crate::{ParallelRunner, orch::Orchestrator};
use std::marker::PhantomData;

pub struct StdOrchestrator<R>
where
    R: ParallelRunner,
{
    r: PhantomData<R>,
}

impl<R> Orchestrator for StdOrchestrator<R>
where
    R: ParallelRunner,
{
    type Runner = R;
}
