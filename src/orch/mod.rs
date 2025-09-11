mod implementations;
mod orchestrator;
mod par_handle;
mod par_scope;

pub use crate::orch::implementations::StdOrchestrator;
pub use orchestrator::Orchestrator;
pub use par_handle::ParHandle;
pub use par_scope::ParScope;

pub type DefaultOrchestrator = StdOrchestrator<crate::DefaultRunner>;
