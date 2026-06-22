use crate::ParUse;
use crate::infallible_use::{ParUseIter, XapUseEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

/// Adds index-value pairs to `ParUse` pipelines.
///
/// This is the `ParUse` counterpart of parallel `enumerate`, preserving the
/// worker-local state while changing each item into `(index, item)`.
///
/// # Example
///
/// ```
/// use orx_parallel::*;
///
/// let values: Vec<_> = (10..15)
///     .into_par()
///     .use_new(|_| ())
///     .enumerate()
///     .map(|_, (i, x)| i + x)
///     .collect();
///
/// assert_eq!(values.len(), 5);
/// assert!(values.contains(&10));
/// assert!(values.contains(&18));
/// ```
pub trait EnumerateParUse: ParUse {
    /// Transforms each item into `(index, item)` while keeping worker-local state.
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
    ///     .use_new(|_| ())
    ///     .enumerate()
    ///     .map(|_, (i, _)| i)
    ///     .sum();
    ///
    /// assert_eq!(idx_sum, 10);
    /// ```
    fn enumerate(
        self,
    ) -> impl ParUse<
        Using = Self::Using,
        Use = Self::Use,
        Item = (usize, Self::Item),
        Input = Enumerate<Self::Input>,
    >;
}

impl<P> EnumerateParUse for P
where
    P: ParUse,
    P::Xap: XapUseEnumByInput,
{
    fn enumerate(
        self,
    ) -> impl ParUse<
        Using = Self::Using,
        Use = Self::Use,
        Item = (usize, Self::Item),
        Input = Enumerate<Self::Input>,
    >
    where
        Self::Xap: XapUseEnumByInput,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParUseIter::new(u, iter, xap, exe, params)
    }
}
