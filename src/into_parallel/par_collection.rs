use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::{ConcurrentCollection, ConcurrentIterable};

/// Adds `.par()` to concurrent collections.
///
/// Sequential counterpart: collection iteration methods such as `iter()`.
pub trait ParCollection: ConcurrentCollection {
    /// Returns a parallel iterator over shared references to collection items.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let values = vec![1, 2, 3, 4];
    /// let sum: i32 = ParCol::par(&values).copied().sum();
    ///
    /// assert_eq!(sum, 10);
    /// ```
    fn par(&self) -> ParIter<<Self::Iterable<'_> as ConcurrentIterable>::Iter, Id<&Self::Item>> {
        ParIter::new(
            self.con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<X> ParCollection for X where X: ConcurrentCollection {}
