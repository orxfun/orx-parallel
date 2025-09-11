use crate::{Params, computational_variants::Par, orch::DefaultOrchestrator};
use orx_concurrent_iter::{ConcurrentCollection, ConcurrentIterable};

/// A type implementing [`ParallelizableCollection`] is a collection owning the elements such that
///
/// * if the elements are of type `T`,
/// * non-consuming [`par`] method can be called **multiple times** to create parallel
///   iterators; i.e., [`ParIter`], yielding references to the elements `&T`; and
/// * consuming [`into_par`] method can be called once to create a parallel iterator yielding
///   owned elements `T`.
///
/// This trait can be considered as the *concurrent counterpart* of the [`Collection`] trait.
///
/// Note that every [`ConcurrentCollection`] type automatically implements [`ParallelizableCollection`].
///
/// [`Collection`]: orx_iterable::Collection
/// [`ConcurrentCollection`]: orx_concurrent_iter::ConcurrentCollection
/// [`par`]: crate::ParallelizableCollection::par
/// [`into_par`]: crate::IntoParIter::into_par
/// [`ParIter`]: crate::ParIter
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // Vec<T>: ParallelizableCollection<Item = T>
/// let vec = vec![1, 2, 3, 4];
///
/// // non-consuming iteration over references
/// assert_eq!(vec.par().sum(), 10);
/// assert_eq!(vec.par().min(), Some(&1));
/// assert_eq!(vec.par().max(), Some(&4));
///
/// // consuming iteration over owned values
/// assert_eq!(vec.into_par().max(), Some(4));
/// ```
pub trait ParallelizableCollection: ConcurrentCollection {
    /// A type implementing [`ParallelizableCollection`] is a collection owning the elements such that
    ///
    /// * if the elements are of type `T`,
    /// * non-consuming [`par`] method can be called **multiple times** to create parallel
    ///   iterators; i.e., [`ParIter`], yielding references to the elements `&T`; and
    /// * consuming [`into_par`] method can be called once to create a parallel iterator yielding
    ///   owned elements `T`.
    ///
    /// This trait can be considered as the *concurrent counterpart* of the [`Collection`] trait.
    ///
    /// Note that every [`ConcurrentCollection`] type automatically implements [`ParallelizableCollection`].
    ///
    /// [`con_iter`]: orx_concurrent_iter::ConcurrentCollection::con_iter
    /// [`Collection`]: orx_iterable::Collection
    /// [`ConcurrentIter`]: orx_concurrent_iter::ConcurrentIter
    /// [`par`]: crate::ParallelizableCollection::par
    /// [`into_par`]: crate::IntoParIter::into_par
    /// [`ParIter`]: crate::ParIter
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // Vec<T>: ParallelizableCollection<Item = T>
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // non-consuming iteration over references
    /// assert_eq!(vec.par().sum(), 10);
    /// assert_eq!(vec.par().min(), Some(&1));
    /// assert_eq!(vec.par().max(), Some(&4));
    ///
    /// // consuming iteration over owned values
    /// assert_eq!(vec.into_par().max(), Some(4));
    /// ```
    fn par(
        &self,
    ) -> Par<
        <<Self as ConcurrentCollection>::Iterable<'_> as ConcurrentIterable>::Iter,
        DefaultOrchestrator,
    > {
        Par::new(Default::default(), Params::default(), self.con_iter())
    }
}

impl<X> ParallelizableCollection for X where X: ConcurrentCollection {}
