use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::ConcurrentIterable;

/// Adds `.par()` to concurrent iterables.
///
/// Sequential counterpart: creating and using a regular iterator via `iter()` or `into_iter()`.
pub trait Parallelizable: ConcurrentIterable {
    /// Returns a parallel iterator over items of this iterable.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: usize = (0..10).par().sum();
    /// assert_eq!(sum, 45);
    /// ```
    fn par(&self) -> ParIter<Self::Iter, Id<Self::Item>> {
        ParIter::new(
            self.con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<I> Parallelizable for I where I: ConcurrentIterable {}
