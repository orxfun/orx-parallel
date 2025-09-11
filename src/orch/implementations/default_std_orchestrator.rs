use crate::{
    DefaultRunner,
    orch::{Orchestrator, thread_pool::implementations::StdDefaultThreadPool},
};

#[derive(Default)]
pub struct DefaultStdOrchestrator(StdDefaultThreadPool);

impl Orchestrator for DefaultStdOrchestrator {
    type Runner = DefaultRunner;

    type ThreadPool = StdDefaultThreadPool;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.0
    }
}
