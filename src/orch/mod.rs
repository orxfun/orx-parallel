mod implementations;
mod num_spawned;
mod orchestrator;
mod par_thread_pool;

pub(crate) use orchestrator::{SharedStateOf, ThreadRunnerOf};

pub use crate::orch::implementations::DefaultStdOrchestrator;
pub use num_spawned::NumSpawned;
pub use orchestrator::Orchestrator;
pub use par_thread_pool::{ParThreadPool, ParThreadPoolCompute};

pub type DefaultOrchestrator = DefaultStdOrchestrator;
