mod computation_kind;
mod map_collect;
mod runner;

pub(crate) use map_collect::MapCollect;

pub use runner::{DefaultRunner, ParallelRunner};
