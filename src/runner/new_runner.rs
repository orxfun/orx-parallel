use crate::ParThreadPool;
use crate::runner::runner_variants::{DynChunkRunner, FixedChunkRunner};

pub struct Runner;

impl Runner {
    pub fn fixed_chunk<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
        FixedChunkRunner::new(pool)
    }

    pub fn dynamic_chunk<P: ParThreadPool>(pool: P) -> DynChunkRunner<P> {
        DynChunkRunner::new(pool)
    }
}
