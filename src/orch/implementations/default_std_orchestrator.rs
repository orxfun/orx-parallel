use crate::{
    DefaultRunner,
    orch::{Orchestrator, thread_pool::implementations::StdDefaultPool},
};

#[derive(Default)]
pub struct DefaultStdOrchestrator(StdDefaultPool);

impl Orchestrator for DefaultStdOrchestrator {
    type Runner = DefaultRunner;

    type ThreadPool = StdDefaultPool;

    fn thread_pool(&mut self) -> &Self::ThreadPool {
        &self.0
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.0
    }
}
