use crate::Par;
use crate::infallible::{ParIter, XapEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

/// Adds index-value pairs to infallible parallel iterators.
///
/// This is the parallel counterpart of [`Iterator::enumerate`](core::iter::Iterator::enumerate).
///
/// # Example
///
/// ```
/// use orx_parallel::*;
///
/// let pairs: Vec<_> = (10..15)
///     .into_par()
///     .enumerate()
///     .collect();
///
/// assert_eq!(pairs.len(), 5);
/// assert!(pairs.contains(&(0, 10)));
/// assert!(pairs.contains(&(4, 14)));
/// ```
pub trait EnumeratePar: Par {
    /// Transforms each item into `(index, item)`.
    ///
    /// Indices are zero-based and correspond to the iterator order.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let idx_sum: usize = (1..6)
    ///     .into_par()
    ///     .enumerate()
    ///     .map(|(i, _)| i)
    ///     .sum();
    ///
    /// assert_eq!(idx_sum, 10);
    /// ```
    fn enumerate(self) -> impl Par<Item = (usize, Self::Item), Input = Enumerate<Self::Input>>;
}

impl<P> EnumeratePar for P
where
    P: Par,
    P::Xap: XapEnumByInput,
{
    fn enumerate(self) -> impl Par<Item = (usize, Self::Item), Input = Enumerate<Self::Input>>
    where
        Self::Xap: XapEnumByInput,
    {
        let (iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParIter::new(iter, xap, exe, params)
    }
}
