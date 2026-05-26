use crate::ParThreadPool;
#[cfg(feature = "std")]
use crate::runner::runner_variants::DynChunkRunner;
use crate::runner::runner_variants::FixedChunkRunner;

pub struct Runner;

impl Runner {
    pub fn fixed_chunk<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
        FixedChunkRunner::new(pool)
    }

    #[cfg(feature = "std")]
    pub fn dynamic_chunk<P: ParThreadPool>(pool: P) -> DynChunkRunner<P> {
        DynChunkRunner::new(pool)
    }
}
