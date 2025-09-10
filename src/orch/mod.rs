mod implementations;
mod orchestrator;

pub use crate::orch::implementations::StdOrchestrator;

pub type DefaultOrchestrator = StdOrchestrator<crate::DefaultRunner>;
