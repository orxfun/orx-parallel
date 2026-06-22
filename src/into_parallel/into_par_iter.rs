use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::IntoConcurrentIter;

/// Converts values into a parallel iterator.
///
/// Sequential counterpart: [`IntoIterator::into_iter`](core::iter::IntoIterator::into_iter).
pub trait IntoParIter: IntoConcurrentIter {
    /// Consumes `self` and returns a parallel iterator over its items.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: i32 = vec![1, 2, 3, 4]
    ///     .into_par()
    ///     .map(|x| x * 2)
    ///     .sum();
    ///
    /// assert_eq!(sum, 20);
    /// ```
    fn into_par(self) -> ParIter<Self::IntoIter, Id<Self::Item>>;
}

impl<I: IntoConcurrentIter> IntoParIter for I {
    fn into_par(self) -> ParIter<Self::IntoIter, Id<Self::Item>> {
        ParIter::new(
            self.into_con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}
