use crate::ParThreadPool;
#[cfg(all(feature = "std", feature = "experimental"))]
use crate::runner::runner_variants::DynChunkRunner;
use crate::runner::runner_variants::FixedChunkRunner;

pub struct Runner;

impl Runner {
    pub fn fixed_chunk<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
        FixedChunkRunner::new(pool)
    }

    #[cfg(all(feature = "std", feature = "experimental"))]
    pub fn dynamic_chunk<P: ParThreadPool>(pool: P) -> DynChunkRunner<P> {
        DynChunkRunner::new(pool)
    }
}
