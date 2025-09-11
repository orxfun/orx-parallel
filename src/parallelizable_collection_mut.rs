use crate::{
    ParIter, ParallelizableCollection, Params, computational_variants::Par,
    orch::DefaultOrchestrator,
};
use orx_concurrent_iter::ConcurrentCollectionMut;

/// A type implementing [`ParallelizableCollectionMut`] is a collection owning the elements such that
///
/// * if the elements are of type `T`,
/// * non-consuming [`par`] method can be called **multiple times** to create parallel
///   iterators; i.e., [`ParIter`], yielding references to the elements `&T`; and
/// * non-consuming [`par_mut`] method can be called **multiple times** to create parallel
///   iterators yielding mutable references to the elements `&mut T`; and
/// * consuming [`into_par`] method can be called once to create a parallel iterator yielding
///   owned elements `T`.
///
/// This trait can be considered as the *concurrent counterpart* of the [`CollectionMut`] trait.
///
/// Note that every [`ConcurrentCollectionMut`] type automatically implements [`ParallelizableCollectionMut`].
///
/// [`CollectionMut`]: orx_iterable::CollectionMut
/// [`ConcurrentCollectionMut`]: orx_concurrent_iter::ConcurrentCollectionMut
/// [`par`]: crate::ParallelizableCollection::par
/// [`par_mut`]: crate::ParallelizableCollectionMut::par_mut
/// [`into_par`]: crate::IntoParIter::into_par
/// [`ParIter`]: crate::ParIter
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // Vec<T>: ParallelizableCollectionMut<Item = T>
/// let mut vec = vec![1, 2, 3, 4];
///
/// // non-consuming iteration over references
/// assert_eq!(vec.par().sum(), 10);
/// assert_eq!(vec.par().min(), Some(&1));
/// assert_eq!(vec.par().max(), Some(&4));
///
/// // non-consuming mutable iteration
/// vec.par_mut().filter(|x| **x >= 3).for_each(|x| *x += 10);
/// assert_eq!(&vec, &[1, 2, 13, 14]);
///
/// // consuming iteration over owned values
/// assert_eq!(vec.into_par().max(), Some(14));
/// ```
pub trait ParallelizableCollectionMut: ConcurrentCollectionMut + ParallelizableCollection {
    /// Creates a parallel iterator over mutable references of the collection's elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // Vec<T>: ParallelizableCollectionMut<Item = T>
    /// let mut vec = vec![1, 2, 3, 4];
    ///
    /// vec.par_mut().filter(|x| **x >= 3).for_each(|x| *x += 10);
    ///
    /// assert_eq!(&vec, &[1, 2, 13, 14]);
    /// ```
    fn par_mut(&mut self) -> impl ParIter<DefaultOrchestrator, Item = &mut Self::Item> {
        Par::new(Default::default(), Params::default(), self.con_iter_mut())
    }
}

impl<X> ParallelizableCollectionMut for X where X: ConcurrentCollectionMut + ParallelizableCollection
{}
