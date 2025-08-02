use crate::{ParIter, Params, computational_variants::Par, runner::DefaultRunner};
use orx_concurrent_iter::IntoConcurrentIter;

/// Trait to convert a source (collection or generator) into a parallel iterator; i.e., [`ParIter`],
/// using its [`into_par`] method.
///
/// It can be considered as the *concurrent counterpart* of the [`IntoIterator`] trait.
///
/// Note that every [`IntoConcurrentIter`] type automatically implements [`IntoParIter`].
///
/// [`into_par`]: crate::IntoParIter::into_par
/// [`IntoConcurrentIter`]: orx_concurrent_iter::IntoConcurrentIter
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // Vec<T>: IntoParIter<Item = T>
/// let vec = vec![1, 2, 3, 4];
/// assert_eq!(vec.into_par().max(), Some(4));
///
/// // Range<T>: IntoParIter<Item = T>
/// let range = 1..5;
/// assert_eq!(range.into_par().max(), Some(4));
/// ```
pub trait IntoParIter: IntoConcurrentIter {
    /// Trait to convert a source (collection or generator) into a parallel iterator; i.e., [`ParIter`],
    /// using its [`into_par`] method.
    ///
    /// It can be considered as the *concurrent counterpart* of the [`IntoIterator`] trait.
    ///
    /// [`into_par`]: crate::IntoParIter::into_par
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // Vec<T>: IntoParIter<Item = T>
    /// let vec = vec![1, 2, 3, 4];
    /// assert_eq!(vec.into_par().max(), Some(4));
    ///
    /// // Range<T>: IntoParIter<Item = T>
    /// let range = 1..5;
    /// assert_eq!(range.into_par().max(), Some(4));
    /// ```
    fn into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item>;
}

impl<I> IntoParIter for I
where
    I: IntoConcurrentIter,
{
    fn into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item> {
        Par::new(Params::default(), self.into_con_iter())
    }
}
