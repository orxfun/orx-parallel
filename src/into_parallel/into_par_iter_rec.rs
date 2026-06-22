use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

/// Converts recursive structures into an infallible parallel iterator.
pub trait IntoParIterRecursive
where
    Self: IntoIterator,
    Self::Item: Send,
{
    /// Creates a parallel recursive iterator using `extend` to discover children.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut values: Vec<_> = vec![0usize]
    ///     .into_par_recursive(|x| if *x == 0 { vec![1, 2] } else { Vec::new() })
    ///     .collect();
    ///
    /// values.sort();
    /// assert_eq!(values, vec![0, 1, 2]);
    /// ```
    fn into_par_recursive<I, F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<I, F>, Id<Self::Item>>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync;
}

impl<X> IntoParIterRecursive for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_recursive<I, F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<I, F>, Id<Self::Item>>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync,
    {
        let iter = ConcurrentRecursiveIter::new(self, extend, None, None);
        ParIter::new(iter, Id::new(), default_runner(), Default::default())
    }
}
