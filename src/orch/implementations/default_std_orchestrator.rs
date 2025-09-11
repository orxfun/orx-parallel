use crate::{
    DefaultRunner,
    orch::{Orchestrator, thread_pool::implementations::StdOsThreadPool},
};

#[derive(Default)]
pub struct DefaultStdOrchestrator(StdOsThreadPool);

impl Orchestrator for DefaultStdOrchestrator {
    type Runner = DefaultRunner;

    type ThreadPool = StdOsThreadPool;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.0
    }
}
