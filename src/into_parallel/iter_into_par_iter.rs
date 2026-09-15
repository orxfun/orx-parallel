use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::{IterIntoConcurrentIter, implementations::ConIterOfIter};

/// Converts a regular standard iterator into a parallel iterator.
pub trait IterIntoParIter: Iterator {
    /// Converts `self` into a parallel iterator.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: i32 = (0..10)
    ///     .into_iter()
    ///     .iter_into_par()
    ///     .map(|x| x * 2)
    ///     .sum();
    ///
    /// assert_eq!(sum, 90);
    /// ```
    fn iter_into_par(self) -> ParIter<ConIterOfIter<Self>, Id<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send;
}

impl<I> IterIntoParIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_par(self) -> ParIter<ConIterOfIter<Self>, Id<Self::Item>> {
        ParIter::new(
            self.iter_into_con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}
