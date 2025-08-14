mod collect_arbitrary;
mod collect_ordered;
mod collect_sequential;
mod fallibility;

pub use collect_arbitrary::{ArbitraryPush, ParallelCollectArbitrary, ThreadCollectArbitrary};
pub use collect_ordered::{OrderedPush, ParallelCollect, ThreadCollect};
pub use fallibility::{Fallability, Fallible, Infallible};
