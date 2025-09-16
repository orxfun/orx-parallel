#[cfg(test)]
mod tests;

mod default_std_orchestrator;
pub use default_std_orchestrator::DefaultStdOrchestrator;

#[cfg(feature = "rayon")]
mod rayon;
#[cfg(feature = "rayon")]
pub use rayon::RayonOrchestrator;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;
#[cfg(feature = "scoped_threadpool")]
pub use scoped_threadpool::ScopedThreadPoolOrchestrator;
