mod implementations;
mod orchestrator;
mod par_handle;
mod par_scope;

pub use crate::orch::implementations::StdOrchestrator;
pub use orchestrator::Orchestrator;

pub type DefaultOrchestrator = StdOrchestrator<crate::DefaultRunner>;
