mod computation_kind;
mod implementations;
mod num_spawned;
mod orchestrator;

pub(crate) use orchestrator::{SharedStateOf, ThreadRunnerOf};

pub use crate::orch::implementations::DefaultStdOrchestrator;
pub use computation_kind::ComputationKind;
pub use num_spawned::NumSpawned;
pub use orchestrator::Orchestrator;

pub type DefaultOrchestrator = DefaultStdOrchestrator;
