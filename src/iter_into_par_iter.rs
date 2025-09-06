use crate::{Params, computational_variants::Par, runner::DefaultRunner};
use orx_concurrent_iter::{IterIntoConcurrentIter, implementations::ConIterOfIter};

/// Any regular iterator implements [`IterIntoParIter`] trait allowing them to be used
/// as a parallel iterator; i.e., [`ParIter`], by calling [`iter_into_par`].
///
/// Pulling of elements from the iterator are synchronized and safely shared to threads.
///
/// Therefore, converting an iterator into a parallel iterator is most useful whenever
/// the work to be done on each element is a larger task than just yielding elements by the
/// underlying collection or generator.
///
/// Note that every [`IterIntoConcurrentIter`] type automatically implements [`IterIntoParIter`].
///
/// [`iter_into_par`]: crate::IterIntoParIter::iter_into_par
/// [`ParIter`]: crate::ParIter
/// [`IterIntoConcurrentIter`]: orx_concurrent_iter::IterIntoConcurrentIter
///
/// # Examples
///
/// In the following example, an arbitrary iterator is converted into a parallel iterator
/// and shared with multiple threads as a shared reference.
///
/// ```
/// use orx_parallel::*;
///
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///
/// // an arbitrary iterator
/// let iter = data
///     .into_iter()
///     .filter(|x| !x.starts_with('3'))
///     .map(|x| format!("{x}!"));
///
/// // convert arbitrary iterator into ParIter
/// let par_iter = iter.iter_into_par();
/// let num_characters = par_iter.map(|x| x.len()).sum();
///
/// assert_eq!(num_characters, 258);
/// ```
///
/// Similarly, in the following example, computation over elements of a generic
/// iterator are distributed into multiple threads.
///
/// ```
/// use orx_parallel::*;
///
/// let data: Vec<_> = (0..123).collect();
///
/// // arbitrary iterator
/// let iter = data.iter().filter(|x| *x % 2 == 0).map(|x| x.to_string());
///
/// // parallel computation
/// let sum_evens = iter
///     .iter_into_par()
///     .map(|x| x.parse::<u64>().unwrap())
///     .sum();
///
/// assert_eq!(sum_evens, 3782);
/// ```
pub trait IterIntoParIter: Iterator {
    /// Any regular iterator implements [`IterIntoParIter`] trait allowing them to be used
    /// as a parallel iterator; i.e., [`ParIter`], by calling [`iter_into_par`].
    ///
    /// Pulling of elements from the iterator are synchronized and safely shared to threads.
    ///
    /// Therefore, converting an iterator into a parallel iterator is most useful whenever
    /// the work to be done on each element is a larger task than just yielding elements by the
    /// underlying collection or generator.
    ///
    /// Note that every [`IterIntoConcurrentIter`] type automatically implements [`IterIntoParIter`].
    ///
    /// [`iter_into_par`]: crate::IterIntoParIter::iter_into_par
    /// [`ParIter`]: crate::ParIter
    /// [`IterIntoConcurrentIter`]: orx_concurrent_iter::IterIntoConcurrentIter
    ///
    /// # Examples
    ///
    /// In the following example, an arbitrary iterator is converted into a parallel iterator
    /// and shared with multiple threads as a shared reference.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
    ///
    /// // an arbitrary iterator
    /// let iter = data
    ///     .into_iter()
    ///     .filter(|x| !x.starts_with('3'))
    ///     .map(|x| format!("{x}!"));
    ///
    /// // convert arbitrary iterator into ParIter
    /// let par_iter = iter.iter_into_par();
    /// let num_characters = par_iter.map(|x| x.len()).sum();
    ///
    /// assert_eq!(num_characters, 258);
    /// ```
    ///
    /// Similarly, in the following example, computation over elements of a generic
    /// iterator are distributed into multiple threads.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data: Vec<_> = (0..123).collect();
    ///
    /// // arbitrary iterator
    /// let iter = data.iter().filter(|x| *x % 2 == 0).map(|x| x.to_string());
    ///
    /// // parallel computation
    /// let sum_evens = iter
    ///     .iter_into_par()
    ///     .map(|x| x.parse::<u64>().unwrap())
    ///     .sum();
    ///
    /// assert_eq!(sum_evens, 3782);
    /// ```
    fn iter_into_par(self) -> Par<ConIterOfIter<Self>, DefaultRunner>
    where
        Self: Sized,
        Self::Item: Send;
}

impl<I> IterIntoParIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_par(self) -> Par<ConIterOfIter<Self>, DefaultRunner> {
        Par::new(Params::default(), self.iter_into_con_iter())
    }
}
