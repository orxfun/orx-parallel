mod implementations;
mod num_spawned;
mod orchestrator;
mod thread_pool;

pub use crate::orch::implementations::DefaultStdOrchestrator;
pub use num_spawned::NumSpawned;
pub use orchestrator::Orchestrator;
pub use thread_pool::{ParHandle, ParScope, ParThreadPool};

pub type DefaultOrchestrator = DefaultStdOrchestrator;
