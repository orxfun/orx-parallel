use crate::{computational_variants::Par, orch::DefaultOrchestrator, parameters::Params};
use orx_concurrent_iter::ConcurrentIterable;

/// `Parallelizable` types are those from which parallel iterators can be created
/// **multiple times** using the [`par`] method, since this method call does not consume the source.
///
/// This trait can be considered as the *parallel counterpart* of the [`Iterable`] trait.
///
/// Note that every [`ConcurrentIterable`] type automatically implements [`Parallelizable`].
///
/// [`par`]: crate::Parallelizable::par
/// [`Iterable`]: orx_iterable::Iterable
/// [`ConcurrentIterable`]: orx_concurrent_iter::ConcurrentIterable
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // Vec<T>: Parallelizable<Item = &T>
/// let vec = vec![1, 2, 3, 4];
/// assert_eq!(vec.par().sum(), 10);
/// assert_eq!(vec.par().max(), Some(&4));
///
/// // &[T]: Parallelizable<Item = &T>
/// let slice = vec.as_slice();
/// assert_eq!(slice.par().sum(), 10);
/// assert_eq!(slice.par().max(), Some(&4));
///
/// // Range<T>: Parallelizable<Item = T>
/// let range = 1..5;
/// assert_eq!(range.par().sum(), 10);
/// assert_eq!(range.par().max(), Some(4));
/// ```
pub trait Parallelizable: ConcurrentIterable {
    /// `Parallelizable` types are those from which parallel iterators can be created
    /// **multiple times** using the [`par`] method, since this method call does not consume the source.
    ///
    /// This trait can be considered as the *parallel counterpart* of the [`Iterable`] trait.
    ///
    /// [`par`]: crate::Parallelizable::par
    /// [`Iterable`]: orx_iterable::Iterable
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // Vec<T>: Parallelizable<Item = &T>
    /// let vec = vec![1, 2, 3, 4];
    /// assert_eq!(vec.par().sum(), 10);
    /// assert_eq!(vec.par().max(), Some(&4));
    ///
    /// // &[T]: Parallelizable<Item = &T>
    /// let slice = vec.as_slice();
    /// assert_eq!(slice.par().sum(), 10);
    /// assert_eq!(slice.par().max(), Some(&4));
    ///
    /// // Range<T>: Parallelizable<Item = T>
    /// let range = 1..5;
    /// assert_eq!(range.par().sum(), 10);
    /// assert_eq!(range.par().max(), Some(4));
    /// ```
    fn par(&self) -> Par<<Self as ConcurrentIterable>::Iter, DefaultOrchestrator> {
        Par::new(Params::default(), self.con_iter())
    }
}

impl<I> Parallelizable for I where I: ConcurrentIterable {}
