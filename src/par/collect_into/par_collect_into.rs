use super::collect_into_core::ParCollectIntoCore;

/// Trait representing collections which can be filled which has the concurrent safety to enable collecting results of a parallel computation.
///
/// Some common example collections are:
/// * `std::vec::Vec`
/// * [`SplitVec`](https://crates.io/crates/orx-split-vec)
/// * [`FixedVec`](https://crates.io/crates/orx-fixed-vec)
pub trait ParCollectInto<O: Send + Sync>: ParCollectIntoCore<O> {}
