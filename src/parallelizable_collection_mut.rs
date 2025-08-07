use crate::{
    DefaultRunner, ParIter, ParallelizableCollection, Params, computational_variants::Par,
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
/// [`con_iter`]: orx_concurrent_iter::ConcurrentCollection::con_iter
/// [`CollectionMut`]: orx_iterable::CollectionMut
/// [`ConcurrentCollectionMut`]: orx_concurrent_iter::ConcurrentCollectionMut
/// [`par`]: crate::ParallelizableCollection::par
/// [`par_mut`]: crate::ParallelizableCollectionMut::par_mut
/// [`into_par`]: crate::IntoParIter::into_par
/// [`ParIter`]: crate::ParIter
pub trait ParallelizableCollectionMut: ConcurrentCollectionMut + ParallelizableCollection {
    fn par_mut(&mut self) -> impl ParIter<DefaultRunner, Item = &mut Self::Item> {
        Par::new(Params::default(), self.con_iter_mut())
    }
}

impl<X> ParallelizableCollectionMut for X where X: ConcurrentCollectionMut + ParallelizableCollection
{}
