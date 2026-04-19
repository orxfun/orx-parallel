use crate::infallible_use::Use;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};
use crate::{option::SizePairOpt, runner::ParRunner};

pub trait ParUseOptIter: Sized {
    type Runner: ParRunner;

    type Size: SizePairOpt;

    type Use: Use;

    type Item;
}
